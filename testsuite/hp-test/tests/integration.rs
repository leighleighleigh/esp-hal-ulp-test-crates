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
            reprogram_ulp_core,
            reprogram_ulp_core_with_run_hook,
            ulp_has_booted,
            ulp_is_looping,
            ulp_riscv_halt,
            ulp_riscv_reset,
            ulp_riscv_timer_resume,
            ulp_riscv_timer_stop,
            LpCore,
            LpCorePeripheral,
            LpCoreTimerCycles,
            LpCoreWakeupSource,
        },
    };
    use shared::{
        SharedType,
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

    fn _ulp_test_runner_with_command(core: &mut LpCore, command: UlpCommand) {
        let ulp_wake_src: LpCoreWakeupSource = match command {
            UlpCommand::TIMER_COUNTER_TEST => {
                // LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(53)) // 10 Hz
                LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(5)) // +100 Hz
            }
            _ => LpCoreWakeupSource::HpCpu,
        };
        reprogram_ulp_core(core, ulp_wake_src, command);
    }

    fn _ulp_reset_to_clean_firmware(ulp_core: &mut LpCore) {
        _ulp_test_runner_with_command(ulp_core, UlpCommand::ONESHOT);
        Delay::new().delay_ms(250);
        // check reply was ok
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);
        // check we only ran once
        let a = UlpLoopCounter::load().count();
        hil_test::assert_eq!(a, 1);
    }

    #[test]
    fn ulp_can_be_stopped_and_resumed(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        _ulp_test_runner_with_command(&mut ulp_core, UlpCommand::TIMER_COUNTER_TEST);
        hil_test::assert!(ulp_has_booted());
        hil_test::assert!(ulp_is_looping());
        ulp_riscv_timer_stop();
        ulp_riscv_halt(); // esp-idf tests do a halt here, unsure why...
        hil_test::assert!(!ulp_is_looping());
        ulp_riscv_timer_resume();
        hil_test::assert!(ulp_is_looping());
        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn ulp_can_start_once(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        _ulp_test_runner_with_command(&mut ulp_core, UlpCommand::ONESHOT);
        hil_test::assert_eq!(true, ulp_has_booted());
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);
        let a = UlpLoopCounter::load().count();
        hil_test::assert_eq!(a, 1);
        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn ulp_loop_counter(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        _ulp_test_runner_with_command(&mut ulp_core, UlpCommand::LOOP_COUNTER_TEST);
        hil_test::assert_eq!(true, ulp_has_booted());
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);
        hil_test::assert_eq!(true, ulp_is_looping());
        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn ulp_timer_counter(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        _ulp_test_runner_with_command(&mut ulp_core, UlpCommand::TIMER_COUNTER_TEST);
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);
        defmt::info!("count: {}", UlpLoopCounter::load().count());
        // Delay for a second
        Delay::new().delay_ms(1000);
        // Check the count is above 10
        let count = UlpLoopCounter::load().count();
        defmt::info!("count: {}", count);
        hil_test::assert!(count >= 10);
        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn ulp_can_change_timer_period(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(
            &mut ulp_core,
            // Set the timer to a known ~1Hz period
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(530)),
            || {
                UlpLoopCounter::reset();
                UlpCommand::TIMER_PERIOD_TEST.store();
                UlpReply::UNKNOWN.store();
                // But ask the ULP to configure a faster rate.
                unsafe {
                    ULP_TEST_DATA_IN = 1; // Fast!
                }
            },
        );
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);

        defmt::info!("Waiting for 1 second...");
        // Check its running at a faster rate.
        Delay::new().delay_ms(1000);
        let count = UlpLoopCounter::load().count();
        defmt::info!("count: {}", count);
        hil_test::assert!(count >= 10);

        // Pause timer, ask it to reconfigure for slow rate
        defmt::info!("Pausing timer...");
        ulp_riscv_timer_stop();
        unsafe {
            ULP_TEST_DATA_IN = 530 / 2; // 2Hz
        }
        UlpLoopCounter::reset();
        ulp_riscv_timer_resume();
        defmt::info!("Resuming timer...");
        hil_test::assert!(ulp_has_booted());
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);
        defmt::info!("Waiting for 1 second...");
        Delay::new().delay_ms(1000);
        let count = UlpLoopCounter::load().count();
        defmt::info!("count: {}", count);
        hil_test::assert!(count >= 1 && count <= 3);

        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn ulp_xor_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        let test_value = 0xff;
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpLoopCounter::reset();
            UlpCommand::XOR_TEST.store();
            UlpReply::UNKNOWN.store();
            unsafe { ULP_TEST_DATA_IN = test_value };
        });

        Delay::new().delay_ms(100);
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);
        let result = unsafe { ULP_TEST_DATA_OUT.clone() };
        hil_test::assert_eq!(test_value ^ shared::TEST_XOR_MASK, result);
        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn ulp_stop_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(
            &mut ulp_core,
            LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(5)),
            || {
                UlpLoopCounter::reset();
                UlpCommand::STOP_TEST.store();
                UlpReply::UNKNOWN.store();
            },
        );
        hil_test::assert_eq!(true, ulp_has_booted());
        hil_test::assert_eq!(false, ulp_is_looping());
        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn ulp_mutex_test(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        reprogram_ulp_core_with_run_hook(&mut ulp_core, LpCoreWakeupSource::HpCpu, || {
            UlpLoopCounter::reset();
            UlpCommand::MUTEX_TEST.store();
            UlpReply::UNKNOWN.store();
        });
        hil_test::assert_eq!(true, ulp_has_booted());

        for _ in 0..TEST_MUTEX_ITERATIONS {
            UlpLock::acquire();
            UlpLoopCounter::increment();
            UlpLock::release();
        }

        while UlpReply::load() != UlpReply::OK {
            // Need a delay here, else CPU will block the ULP core.
            Delay::new().delay_micros(1);
        }
        hil_test::assert_eq!(UlpReply::load(), UlpReply::OK);

        // Assert no race conditions and we incremented 2x the number of loops
        hil_test::assert_eq!(2 * TEST_MUTEX_ITERATIONS, UlpLoopCounter::load().count());

        _ulp_reset_to_clean_firmware(&mut ulp_core);
    }

    #[test]
    fn creating_peripheral_does_not_break_debug_connection(ctx: Context) {
        use esp_hal::usb_serial_jtag::UsbSerialJtag;
        _ = UsbSerialJtag::new(ctx.p.USB_DEVICE).into_async().split();
    }
}
