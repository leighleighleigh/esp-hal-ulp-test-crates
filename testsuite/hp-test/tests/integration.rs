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
        i2c::rtc,
        load_lp_code,
        peripherals::{self, Peripherals},
        rtc_cntl::sleep::{self, RtcSleepConfig},
        system::SleepSource,
        time::Instant,
    };
    use hil_test::{
        self as _,
        ulp_debug::{self, FromRegister},
        ulp_utils::{
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
        },
    };
    use semihosting::sys::arm_compat::syscall::{self, ParamRegR, ParamRegW};
    use shared::{
        HpSleepWakeCounter,
        HpSleepWakeupCause,
        SharedType,
        UlpBootCounter,
        UlpCommand,
        UlpHaltCounter,
        UlpLock,
        UlpLoopCounter,
        UlpReply,
        HP_SLEEP_WAKEUP_COUNTER,
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
    fn ulp_mutex_lock_test(ctx: Context) {
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

    // Convert a u32 value to a SleepSource enum
    fn convert_wakeup_cause(value: u32) -> SleepSource {
        match value {
            1 => SleepSource::All,
            2 => SleepSource::Ext0,
            3 => SleepSource::Ext1,
            4 => SleepSource::Timer,
            5 => SleepSource::TouchPad,
            6 => SleepSource::Ulp,
            7 => SleepSource::Gpio,
            8 => SleepSource::Uart,
            9 => SleepSource::Wifi,
            10 => SleepSource::Cocpu,
            11 => SleepSource::CocpuTrapTrig,
            12 => SleepSource::BT,
            _ => SleepSource::Undefined,
        }
    }

    // ulp_light_sleep_wakeup test will be run multiple times,
    // and its behaviour is dependent on the value of HpSleepWakeCounter.
    // It is assumed that OTHER test cases will re-set the valueof HpSleepWakeCounter to zero.
    #[test]
    fn ulp_light_sleep_wakeup(ctx: Context) {
        // Need to save and restore the sleep counter,
        // as starting the ULP will erase the chip.
        let wakeup_count = HpSleepWakeCounter::load();
        let wakeup_cause = convert_wakeup_cause(HpSleepWakeupCause::load());

        defmt::debug!(
            "wakeup_count: {}, wakeup_cause: {}",
            wakeup_count,
            wakeup_cause
        );

        // If wakeup count is zero, we will:
        // 1. Program the ULP core to run the light sleep test
        // 2. Enter light sleep
        // 3. On wake, increment the wakeup count, and store the wakeup cause.
        if wakeup_count == 0 {
            // Program and start the ULP core.
            // For the LIGHT_SLEEP_TEST command, the UlpCore will
            // 1. Increment the loop counter
            // 2. Reply OK to the command
            // 3. Blocking delay for 3 seconds
            // 4. Trigger the 'wake_hp_core' function.
            // 5. Exit and halt.
            let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
            reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
                UlpCommand::LIGHT_SLEEP_TEST.store();
                HpSleepWakeCounter::store(wakeup_count.into());
                HpSleepWakeupCause::store((wakeup_cause as u32).into());
            });
            hil_test::assert!(ulp_has_booted());
            hil_test::assert_eq!(UlpReply::OK, UlpReply::load());
            defmt::debug!("ULP booted.");

            // We now have 3 seconds to enter light sleep,
            // until the ULP core will try to wake us up.
            let wakeup_timer_duration_millis = 5000; // ULP should wake us up before this.

            let mut rtc = esp_hal::rtc_cntl::Rtc::new(ctx.p.LPWR);
            let wakeup_timer = sleep::TimerWakeupSource::new(core::time::Duration::from_millis(
                wakeup_timer_duration_millis,
            ));
            let wakeup_ulp = sleep::WakeFromUlpCoreWakeupSource::new();
            let sleep_config = RtcSleepConfig::default();

            defmt::debug!("Entering light sleep. Probe will disconnect.");
            Delay::new().delay_ms(250);

            // TODO: Tell probe-rs to disconnect cleanly,
            // and to re-start the test again.

            // Enter light sleep, recording the time of entry.
            let sleep_start_timestamp = Instant::now();
            rtc.sleep(&sleep_config, &[&wakeup_timer, &wakeup_ulp]);
            core::mem::drop(rtc);
            // Light sleep will resume here!
            let sleep_duration = sleep_start_timestamp.elapsed().as_millis();
            // Increment the persistent variables in memory, on waking.
            HpSleepWakeCounter::increment();
            // let wake_reason = esp_hal::system::wakeup_cause();
            // HpSleepWakeupCause::store((wake_reason as u32).into());

            // ESP32S3 cannot use wakeup_cause() to detect light-sleep wake-ups.
            // Instead, I'll use the elapsed sleep time to determine the cause.
            // If sleep_duration >= sleep_timer_wakeup_duration, then the ULP failed to wake up the
            // HP core, and our wake_reason is 'Timer'.
            // Else, our wake_reason is 'Ulp'.
            if sleep_duration >= wakeup_timer_duration_millis {
                HpSleepWakeupCause::store((SleepSource::Timer as u32).into());
            } else {
                HpSleepWakeupCause::store((SleepSource::Ulp as u32).into());
            }
        } else {
            // If the wakeup count is non-zero, it means the HP core did wake-up from the light
            // sleep.

            // Check that the ULP core only ran once
            hil_test::assert_eq!(1, UlpHaltCounter::load());

            // Assert that the wakeup cause was due to ULP interrupt.
            match wakeup_cause {
                SleepSource::Ulp => {
                    hil_test::assert!(true);
                }
                _ => {
                    hil_test::assert!(false);
                }
            }
        }
    }

    #[test]
    fn creating_peripheral_does_not_break_debug_connection(ctx: Context) {
        use esp_hal::usb_serial_jtag::UsbSerialJtag;
        _ = UsbSerialJtag::new(ctx.p.USB_DEVICE).into_async().split();
    }
}
