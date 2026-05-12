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
    use esp_hal::{delay::Delay, load_lp_code, peripherals::Peripherals, time::Instant};
    use hil_test::{
        self as _,
        ulp_debug,
        ulp_debug::FromRegister,
        ulp_utils::{
            reprogram_ulp_core,
            ulp_is_running,
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
    use shared::{UlpCommand, UlpCommandType, UlpLoopCounter, UlpReply, UlpReplyType};

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

        Context { p: peripherals }
    }

    fn _ulp_test_runner_(core: &mut LpCore, command: UlpCommandType) {
        let ulp_wake_src: LpCoreWakeupSource = match command {
            UlpCommandType::TIMER_COUNTER_TEST => {
                LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(53))
            }
            _ => LpCoreWakeupSource::HpCpu,
        };
        reprogram_ulp_core(core, ulp_wake_src, command);
    }

    fn ulp_can_be_stopped_and_resumed(ulp_core: &mut LpCore) {
        defmt::println!("ulp_can_be_stopped_and_resumed()");
        // run the timer loop
        _ulp_test_runner_(ulp_core, UlpCommandType::TIMER_COUNTER_TEST);
        // debug the core
        let dbg = ulp_debug::CocpuDebug::read();
        defmt::println!("{:?}", dbg);
        hil_test::assert!(ulp_is_running());
        ulp_riscv_timer_stop();
        // ulp_riscv_halt();
        hil_test::assert!(!ulp_is_running());
        ulp_riscv_timer_resume();
        hil_test::assert!(ulp_is_running());
    }

    fn ulp_can_start_once(ulp_core: &mut LpCore) {
        defmt::println!("ulp_can_start_once()");
        _ulp_test_runner_(ulp_core, UlpCommandType::NOOP);
        hil_test::assert_eq!(true, ulp_is_running());
        hil_test::assert_eq!(UlpReply::read(), UlpReplyType::OK);
        let a = UlpLoopCounter::read();
        hil_test::assert_eq!(a, 1);
    }

    fn ulp_loop_counter(ulp_core: &mut LpCore) {
        defmt::println!("ulp_loop_counter()");
        _ulp_test_runner_(ulp_core, UlpCommandType::LOOP_COUNTER_TEST);
        hil_test::assert_eq!(true, ulp_is_running());
        hil_test::assert_eq!(UlpReply::read(), UlpReplyType::OK);
    }

    fn ulp_timer_counter(ulp_core: &mut LpCore) {
        defmt::println!("ulp_timer_counter()");
        _ulp_test_runner_(ulp_core, UlpCommandType::TIMER_COUNTER_TEST);
        // print debug info for the ulp core
        // let dbg = ulp_debug::CocpuDebug::read();
        // defmt::println!("{:?}", dbg);
        // match dbg.decode_instruction() {
        //     Ok(i) => {
        //         defmt::println!("{:?}", defmt::Debug2Format(&i));
        //     }
        //     Err(e) => {
        //         defmt::println!("{:?}", defmt::Debug2Format(&e));
        //     }
        // }
        hil_test::assert!(ulp_is_running());
        hil_test::assert_eq!(UlpReply::read(), UlpReplyType::OK);
    }

    #[test]
    fn ulp_tests(ctx: Context) {
        let mut ulp_core = LpCore::new(ctx.p.ULP_RISCV_CORE);
        ulp_can_start_once(&mut ulp_core);
        ulp_loop_counter(&mut ulp_core);
        ulp_timer_counter(&mut ulp_core);
        ulp_can_be_stopped_and_resumed(&mut ulp_core);

        drop(ulp_core);
    }

    #[test]
    fn creating_peripheral_does_not_break_debug_connection(ctx: Context) {
        use esp_hal::usb_serial_jtag::UsbSerialJtag;
        _ = UsbSerialJtag::new(ctx.p.USB_DEVICE).into_async().split();
    }
}
