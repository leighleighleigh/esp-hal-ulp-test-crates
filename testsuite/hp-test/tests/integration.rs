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
    use esp_hal::{delay::Delay, i2c::rtc, load_lp_code, peripherals::Peripherals, time::Instant};
    use hil_test::{
        self as _,
        ulp_debug::{self, FromRegister},
        ulp_utils::{
            reprogram_ulp_core_with_run_hook,
            ulp_has_booted,
            ulp_is_looping,
            ulp_riscv_halt,
            ulp_riscv_reset,
            ulp_riscv_timer_resume,
            ulp_riscv_timer_stop,
            ulp_timer_period,
            LpCore,
            LpCorePeripheral,
            LpCoreTimerCycles,
            LpCoreWakeupSource,
        },
    };
    use shared::{
        SharedType,
        UlpBootCounter,
        UlpCommand,
        UlpLock,
        UlpLoopCounter,
        UlpReply,
        TEST_MUTEX_ITERATIONS,
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
            <esp_hal::peripherals::GPIO2 as esp_hal::gpio::RtcPin>::rtcio_pad_hold(
                &io_reg_en, true,
            );
            <esp_hal::peripherals::GPIO2 as esp_hal::gpio::RtcPinWithResistors>::rtcio_pullup(
                &io_reg_en, true,
            );
        }

        // let dbg = ulp_debug::CocpuDebug::read();
        // defmt::println!("\n{}", dbg);
        // let sar_ctrl = esp_hal::peripherals::SENS::regs();
        // sar_ctrl
        //     .sar_peri_reset_conf()
        //     .write(|w| w.sar_cocpu_reset().set_bit());
        // Delay::new().delay_ms(1);
        // // ulp_timer_period(200);
        // // ulp_riscv_timer_resume();
        // // Delay::new().delay_ms(1);
        // sar_ctrl
        //     .sar_peri_reset_conf()
        //     .write(|w| w.sar_cocpu_reset().clear_bit());
        // Delay::new().delay_ms(1);
        // let dbg = ulp_debug::CocpuDebug::read();
        // defmt::println!("\n{}", dbg);

        Context { p: peripherals }
    }

    // fn do_mini_sleep(lpwr: esp_hal::peripherals::LPWR) {
    //     let mut rtc = esp_hal::rtc_cntl::Rtc::new(lpwr);
    //     let timer = esp_hal::rtc_cntl::sleep::TimerWakeupSource::new(
    //         core::time::Duration::from_micros(1).into(),
    //     );
    //     defmt::info!("Entering light sleep.");
    //     let t0 = Instant::now();
    //     rtc.sleep_light(&[&timer]);
    //     let t1 = Instant::now();
    //     defmt::info!("Slept for: {}", (t1 - t0));
    // }

    // This must be called at the end of every test, to place the Lp core into a safe state.
    // Failing to do this, will cause following tests to fail.
    fn ulp_test_finish(ulp_core: &mut LpCore) {
        reprogram_ulp_core_with_run_hook(ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::NOOP.store();
        });
        hil_test::assert_eq!(ulp_has_booted(), true);
        hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
        let a = UlpBootCounter::load();
        hil_test::assert_eq!(1, a);
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
        ulp_test_finish(&mut ulp_core);
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
        ulp_test_finish(&mut ulp_core);
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
        ulp_test_finish(&mut ulp_core);
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
        ulp_test_finish(&mut ulp_core);
    }

    #[test]
    fn ulp_timer_stop_and_resume(ctx: Context) {
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
        ulp_test_finish(&mut ulp_core);
    }

    #[test]
    fn ulp_can_change_its_timer_period(ctx: Context) {
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

        ulp_test_finish(&mut ulp_core);
    }

    #[test]
    fn ulp_xor_test(ctx: Context) {
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
        ulp_test_finish(&mut ulp_core);
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

        // Now try and resume it from the HP core,
        // which should cause the boot counter to increment again.
        ulp_riscv_timer_resume();
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(2, UlpBootCounter::load());

        ulp_test_finish(&mut ulp_core);
    }

    #[test]
    fn ulp_mutex_lock_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpCommand::MUTEX_TEST.store();
        });
        // hil_test::assert!(ulp_has_booted()); // This delay breaks the test somewhat.

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

        ulp_test_finish(&mut ulp_core);
    }

    #[test]
    fn creating_peripheral_does_not_break_debug_connection(ctx: Context) {
        use esp_hal::usb_serial_jtag::UsbSerialJtag;
        _ = UsbSerialJtag::new(ctx.p.USB_DEVICE).into_async().split();
    }
}
