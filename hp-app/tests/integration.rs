#![no_std]
#![no_main]
#![allow(unused_imports)]
#![allow(static_mut_refs)]

//% CHIPS: esp32c6 esp32s2 esp32s3
//% FEATURES: unstable

#[embedded_test::tests()]
mod tests {
    use critical_section::Mutex;
    use embedded_hal::delay::DelayNs;
    #[cfg(esp32c6)]
    use esp32c6 as pac;
    #[cfg(esp32s2)]
    use esp32s2 as pac;
    #[cfg(esp32s3)]
    use esp32s3 as pac;
    use esp_hal::{
        delay::Delay,
        gpio::lp_io::LowPowerPin,
        interrupt,
        load_lp_code,
        peripherals::{self, Peripherals},
        rtc_cntl::{
            reset_reason,
            sleep::{LowPower, RtcSleepConfig},
            wakeup_cause,
            SocResetReason,
        },
        time::Instant,
    };
    use hil_test::{
        self as _,
        ulp_debug::{self, FromRegister},
        ulp_utils::{
            reprogram_ulp_core_with_rainbow_firmware,
            reprogram_ulp_core_with_run_hook,
            ulp_has_booted,
            ulp_is_looping,
            ulp_riscv_halt,
            ulp_riscv_hard_reset,
            ulp_riscv_reset,
            ulp_riscv_timer_resume,
            ulp_riscv_timer_stop,
            ulp_timer_period,
            LpCore,
            LpCorePeripheral,
            LpCoreTimerCycles,
            LpCoreWakeupSource,
            LpWakeupConfig,
        },
    };
    use semihosting::sys::arm_compat::syscall::{self, ParamRegR, ParamRegW};
    use shared::{
        SharedType,
        UlpBootCounter,
        UlpCommand,
        UlpHaltCounter,
        UlpLock,
        UlpLoopCounter,
        UlpReply,
        HP_SLEEP_WAKEUP_COUNTER,
        TEST_MUTEX_ITERATIONS,
        ULP_DEBUG_ISR_DATA,
        ULP_DEBUG_TRAP_DATA,
        ULP_TEST_DATA_IN,
        ULP_TEST_DATA_OUT,
    };

    struct Context {
        p: Peripherals,
    }

    // This is run on EVERY test case.
    #[init]
    fn init() -> Context {
        let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
        let peripherals = esp_hal::init(config);

        {
            // REQUIRED FOR LEIGHLEIGHLEIGH's CUSTOM DEVBOARD ONLY
            // Turn the power on, and keep it on during sleep using pad hold.
            let mut io_reg_en = unsafe { peripherals.GPIO2.clone_unchecked() };
            let mut reg_enable = esp_hal::gpio::Flex::new(io_reg_en.reborrow());
            reg_enable.apply_output_config(
                &esp_hal::gpio::OutputConfig::default()
                    .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                    .with_pull(esp_hal::gpio::Pull::Up),
            );
            reg_enable.set_high();
            reg_enable.set_pad_hold(true);
        }

        // Do a newline in debug mode, so logs are readable
        defmt::debug!("\n");

        // let dbg = ulp_debug::CocpuDebug::read();
        // defmt::debug!("{}", dbg);

        // Rescue the ULP core from a stuck state
        ulp_riscv_hard_reset();

        Context { p: peripherals }
    }

    #[test]
    fn ulp_can_boot(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::NOOP.store();
        });
        hil_test::assert_eq!(ulp_has_booted(), true);
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        // Booted once
        let a = UlpBootCounter::load();
        hil_test::assert_eq!(1, a);
        // Did not loop
        let b = UlpLoopCounter::load();
        hil_test::assert_eq!(0, b);
    }

    #[test]
    fn ulp_loop_oneshot(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::COUNTER_ONESHOT.store();
        });
        hil_test::assert_eq!(ulp_has_booted(), true);
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        // Booted once
        let a = UlpBootCounter::load();
        hil_test::assert_eq!(1, a);
        // Looped once
        let b = UlpLoopCounter::load();
        hil_test::assert_eq!(1, b);
    }

    #[test]
    fn ulp_loop_counter(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::COUNTER_LOOP.store();
        });
        hil_test::assert_eq!(ulp_has_booted(), true);
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        hil_test::assert_eq!(true, ulp_is_looping());
    }

    #[test]
    fn ulp_timer_counter(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(
            &mut ulp_core,
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(5)),
            || {
                UlpCommand::COUNTER_ULP_TIMER.store();
            },
        );
        hil_test::assert_eq!(ulp_has_booted(), true);
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        defmt::debug!("count: {}", UlpLoopCounter::load());
        // Delay for a second
        Delay::new().delay_ms(1000);
        // Check the count is above 10
        let count = UlpLoopCounter::load();
        defmt::debug!("count: {}", count);
        hil_test::assert!(count >= 10);
    }

    #[test]
    fn ulp_can_load_alternate_firmware(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_rainbow_firmware(
            &mut ulp_core,
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(1)),
            ctx.p.GPIO18,
        );
        Delay::new().delay_ms(1000);
    }

    #[test]
    fn hp_can_pause_ulp_timer(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(
            &mut ulp_core,
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(5)),
            || {
                UlpCommand::COUNTER_ULP_TIMER.store();
            },
        );
        hil_test::assert_eq!(ulp_has_booted(), true);
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        hil_test::assert!(ulp_is_looping());
        ulp_riscv_timer_stop();
        ulp_riscv_halt(); // esp-idf tests do a halt here, unsure why...
        hil_test::assert!(!ulp_is_looping());
        ulp_riscv_timer_resume();
        hil_test::assert!(ulp_is_looping());
    }

    #[test]
    fn hp_can_change_ulp_timer_period(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(
            &mut ulp_core,
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(530)), // Approx 1Hz
            || {
                UlpCommand::COUNTER_ULP_TIMER.store();
            },
        );
        hil_test::assert_eq!(ulp_has_booted(), true);
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        hil_test::assert!(ulp_is_looping());
        // Confirm the slow rate is being used
        let count = UlpLoopCounter::load();
        hil_test::assert!(count <= 10);

        // Change speed to a fast one
        ulp_timer_period(0);
        UlpLoopCounter::reset();
        Delay::new().delay_ms(100);
        let count = UlpLoopCounter::load();
        defmt::debug!("count: {}", count);
        hil_test::assert!(count >= 100);

        // Change speed to a slow one
        ulp_timer_period(1000);
        UlpLoopCounter::reset();
        Delay::new().delay_ms(1000);
        let count = UlpLoopCounter::load();
        defmt::debug!("count: {}", count);
        hil_test::assert!(count <= 2);
    }

    #[test]
    fn ulp_can_change_timer_period(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(
            &mut ulp_core,
            // Set the timer to a known ~1Hz period
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(530)),
            || {
                UlpCommand::TIMER_PERIOD_TEST.store();
                // Using DATA_IN, ask the ULP to configure a much faster rate.
                unsafe {
                    ULP_TEST_DATA_IN = 1;
                }
            },
        );
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        hil_test::assert_eq!(true, ulp_is_looping());

        // Check it is running at a fast rate
        defmt::debug!("Waiting for 1 second...");
        Delay::new().delay_ms(1000);
        let count = UlpLoopCounter::load();
        defmt::debug!("count: {}", count);
        hil_test::assert!(count >= 10);

        // Pause timer, ask it to reconfigure for slow rate
        defmt::debug!("Pausing timer...");
        ulp_riscv_timer_stop();
        Delay::new().delay_ms(10);
        hil_test::assert_eq!(false, ulp_is_looping());
        unsafe {
            ULP_TEST_DATA_IN = 530 / 2; // 2Hz
        }
        UlpLoopCounter::reset();
        UlpBootCounter::reset();
        UlpReply::UNSET.store();

        defmt::debug!("Resuming timer...");
        ulp_riscv_timer_resume();
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());

        // Confirm the slow rate was applied
        defmt::debug!("Waiting for 1 second...");
        Delay::new().delay_ms(1000);
        let count = UlpLoopCounter::load();
        defmt::debug!("count: {}", count);
        hil_test::assert!(count >= 1 && count <= 3);
    }

    #[test]
    fn ipc_xor_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        let test_value = 0xff;
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::XOR_TEST.store();
            // The value for the LP core to XOR
            unsafe { ULP_TEST_DATA_IN = test_value };
        });
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());

        // Check the data
        let result = unsafe { ULP_TEST_DATA_OUT.clone() };
        hil_test::assert_eq!(test_value ^ shared::TEST_XOR_MASK, result);
    }

    #[test]
    fn ulp_exception_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::EXCEPTION_TEST.store();
            unsafe { ULP_TEST_DATA_OUT = 0x0 };
            unsafe { ULP_DEBUG_TRAP_DATA = 0x0 };
            unsafe { ULP_DEBUG_ISR_DATA = 0x0 };
        });
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());

        // Read the debug trap data
        let trap_dbg = unsafe { ULP_DEBUG_TRAP_DATA.clone() };
        defmt::debug!("ULP_DEBUG_TRAP_DATA0 = 0x{:08x}", trap_dbg);

        // Check the exception write 0xdeadbeef to the DATA1 variable
        let result = unsafe { ULP_DEBUG_ISR_DATA.clone() };
        defmt::debug!("ULP_DEBUG_TRAP_DATA1: 0x{:08x}", result);
        hil_test::assert_eq!(0xdeadbeef, result);

        // Check that the exception has caused the ULP to halt
        hil_test::assert_eq!(false, ulp_is_looping());
    }

    #[test]
    fn ulp_interrupt_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::START_INT_TEST.store();
            unsafe { ULP_TEST_DATA_OUT = 0x0 };
            unsafe { ULP_DEBUG_TRAP_DATA = 0x0 };
            unsafe { ULP_DEBUG_ISR_DATA = 0x0 };
        });

        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        hil_test::assert_eq!(true, ulp_is_looping());
        // Print debug registers
        let trap_dbg = unsafe { ULP_DEBUG_TRAP_DATA.clone() };
        defmt::debug!("ULP_DEBUG_TRAP_DATA = 0x{:08x}", trap_dbg);
        let isr_dbg = unsafe { ULP_DEBUG_ISR_DATA.clone() };
        defmt::debug!("ULP_DEBUG_ISR_DATA = 0x{:08x}", isr_dbg);
        // Should have first the MachineExternal interrupt handler,
        // which writes 0xcafebabe
        let result = unsafe { ULP_DEBUG_ISR_DATA.clone() };
        defmt::debug!("interrupt wrote: 0x{:08x}", result);
        hil_test::assert_eq!(0xcafebabe, result);
        // The interrupt should not cause the ULP to lock up or halt
        hil_test::assert_eq!(true, ulp_is_looping());
    }

    #[test]
    fn ulp_gpio_interrupt_test(ctx: Context) {
        // Configure GPIO5 for RTC to use!
        const TOGGLECOUNT: usize = 16;
        const TOGGLEINTERVAL: u32 = 5; // millis
        const RTCPIN: usize = 8;
        const INTTYPE: u8 = 3; // 1 = rising, 2 = falling, 3 = any, 4 = low, 5 = high

        {
            let btn_reg = unsafe { &*pac::RTC_IO::PTR };
            btn_reg.touch_pad(RTCPIN).write(|w| unsafe {
                w.mux_sel()
                    .set_bit()
                    .fun_ie()
                    .set_bit()
                    .rue()
                    .clear_bit()
                    .rde()
                    .clear_bit()
                    .fun_sel()
                    .bits(0)
            });
            // Enable the pin interrupt
            btn_reg
                .pin(RTCPIN)
                .write(|w| unsafe { w.int_type().bits(INTTYPE) });
        }

        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::GPIO_INT_TEST.store();
            unsafe { ULP_TEST_DATA_OUT = 0x0 };
            unsafe { ULP_DEBUG_TRAP_DATA = 0x0 };
            unsafe { ULP_DEBUG_ISR_DATA = 0x0 };
        });

        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        hil_test::assert_eq!(true, ulp_is_looping());

        // The HP core will toggle the pin, and check that the LP core interrupt was fired.
        for i in 0..TOGGLECOUNT {
            let hp_output_lvl = i % 2 == 0;
            let hp_reg = unsafe { &*pac::RTC_IO::PTR };

            // Pin must be set as an output, before we can control it.
            hp_reg
                .rtc_gpio_enable()
                .write(|w| unsafe { w.rtc_gpio_enable().bits(1 << RTCPIN) });

            // toggle the pin output level, using manual register writes.
            if hp_output_lvl {
                hp_reg
                    .rtc_gpio_out_w1ts()
                    .write(|w| unsafe { w.rtc_gpio_out_data_w1ts().bits(1 << RTCPIN) });
            } else {
                hp_reg
                    .rtc_gpio_out_w1tc()
                    .write(|w| unsafe { w.rtc_gpio_out_data_w1tc().bits(1 << RTCPIN) });
            }

            // Tweak this for fun to see how fast it can go
            Delay::new().delay_millis(TOGGLEINTERVAL);

            // Evaluate the result, which depends on INTTYPE
            let lp_trap_data = unsafe { ULP_DEBUG_TRAP_DATA.clone() };
            let lp_isr_data = unsafe { ULP_DEBUG_ISR_DATA.clone() };
            // Now need to clear the interrupt data, for the next iteration.
            unsafe {
                ULP_DEBUG_TRAP_DATA = 0;
                ULP_DEBUG_ISR_DATA = 0;
            }

            let lp_did_interrupt = ((lp_trap_data >> 10) & (1 << RTCPIN)) != 0;
            let lp_input_lvl = ((lp_isr_data >> 10) & (1 << RTCPIN)) != 0;

            defmt::debug!(
                "HP output: {}, LP interrupted: {}, LP input: {}",
                hp_output_lvl,
                lp_did_interrupt,
                lp_input_lvl
            );

            match INTTYPE {
                1 => {
                    // 1 == rising edge, so only check when hp_output_lvl is True
                    if hp_output_lvl {
                        hil_test::assert_eq!(true, lp_did_interrupt);
                        hil_test::assert_eq!(hp_output_lvl, lp_input_lvl);
                    }
                }
                2 => {
                    // 2 == falling edge, so only check when hp_output_lvl is False
                    if !hp_output_lvl {
                        hil_test::assert_eq!(true, lp_did_interrupt);
                        hil_test::assert_eq!(hp_output_lvl, lp_input_lvl);
                    }
                }
                3 => {
                    // 3 == any edge, so we check for every edge!
                    hil_test::assert_eq!(true, lp_did_interrupt);
                    hil_test::assert_eq!(hp_output_lvl, lp_input_lvl);
                }
                _ => {}
            }
        }

        // The interrupt should not cause the ULP to lock up or halt
        hil_test::assert_eq!(true, ulp_is_looping());
    }

    #[test]
    fn ulp_can_stop_itself_then_resumed_by_hp(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(
            &mut ulp_core,
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(5)),
            || {
                UlpCommand::STOP_TEST.store();
            },
        );
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        // It should not be looping if it stopped itself correctly
        hil_test::assert_eq!(false, ulp_is_looping());
        hil_test::assert_eq!(1, UlpBootCounter::load());
        hil_test::assert_eq!(1, UlpHaltCounter::load());

        // Now try and resume it from the HP core,
        // which should cause the boot counter to increment again.
        // The UlpCommand should be the same, so both Boot and Halt counters should increment.
        ulp_riscv_timer_resume();
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(2, UlpBootCounter::load());
        hil_test::assert_eq!(2, UlpHaltCounter::load());
    }

    #[test]
    fn ipc_mutex_lock_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::MUTEX_TEST.store();
        });

        for _ in 0..TEST_MUTEX_ITERATIONS {
            UlpLock::acquire();
            UlpLoopCounter::increment();
            UlpLock::release();
        }

        while UlpReply::load() != UlpReply::OK {
            // Need a delay here, else CPU will block the ULP core.
            Delay::new().delay_micros(100);
        }
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());

        // Assert no race conditions and we incremented 2x the number of loops
        hil_test::assert_eq!(2 * TEST_MUTEX_ITERATIONS, UlpLoopCounter::load());
    }

    #[test]
    fn hp_light_sleep_wakeup_by_ulp(ctx: Context) {
        let mut lpwr = LowPower::new(ctx.p.LPWR);
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);

        // Program and start the ULP core, which will wake us up after 3 seconds.
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::LIGHT_SLEEP_TEST.store();
            UlpHaltCounter::store(0.into());
        });

        // Immediately acquire the lock, before the ULP core does,
        // and release it when we want to be woken up.
        UlpLock::acquire();
        defmt::debug!("UlpLock acquired.");

        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        defmt::debug!("ULP booted.");

        // Wait 0.1 seconds (maximum time) for ULP to be ready to handle the lock
        Delay::new().delay_ms(100);

        // The core is allowed to wake us up
        ulp_core.enable_wakeup(LpWakeupConfig::default());
        // If we aren't woken within 3 seconds, the timer will wake us up.
        let wakeup_deadline = esp_hal::time::Duration::from_millis(3000);
        lpwr.set_wakeup_deadline(Instant::now() + wakeup_deadline);
        defmt::debug!("Entering light sleep...");

        // Enter light sleep, recording the time of entry.
        let sleep_cfg = RtcSleepConfig::default();
        let sleep_start_timestamp = Instant::now();
        UlpLock::release();
        lpwr.sleep_light(sleep_cfg);

        /////////////////////////////////////////////////////////////////////////////////

        // Light sleep will resume here!
        let sleep_duration = sleep_start_timestamp.elapsed();
        defmt::debug!("slept for {}", sleep_duration);

        // Use the elapsed sleep time to determine PASS/FAIL

        // If sleep_duration >= sleep_timer_wakeup_duration,
        // then the ULP failed to wake us up, before the timer did.
        hil_test::assert!(sleep_duration < wakeup_deadline);

        // Check that the ULP core only ran once
        hil_test::assert_eq!(UlpHaltCounter::load(), 1);
    }

    #[test]
    fn creating_peripheral_does_not_break_debug_connection(ctx: Context) {
        use esp_hal::usb::usb_serial_jtag::UsbSerialJtag;
        _ = UsbSerialJtag::new(ctx.p.USB_DEVICE).into_async().split();
    }
}
