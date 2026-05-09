#![no_std]
#![no_main]
#![allow(unused_imports)]

//% CHIPS: esp32c6 esp32s2 esp32s3
//% FEATURES: unstable

#[embedded_test::tests()]
mod tests {
    use embedded_hal::delay::DelayNs;
    use esp_hal::{delay::Delay, peripherals::Peripherals, time::Instant};
    use hil_test::{
        self as _,
        ulp_debug,
        ulp_debug::FromRegister,
        ulp_utils::{
            LpCorePeripheral,
            LpCoreTimerCycles,
            LpCoreWakeupSource,
            erase_ulp_core,
            start_ulp_core,
            ulp_is_running,
            ulp_riscv_halt,
            ulp_riscv_reset,
            ulp_riscv_timer_resume,
            ulp_riscv_timer_stop,
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

        Context { p: peripherals }
    }

    // Halt ULP, set command, and start ULP
    fn _ulp_test_runner(core: LpCorePeripheral, command: UlpCommandType) {
        let ulp_wake_src: LpCoreWakeupSource = match command {
            UlpCommandType::TIMER_COUNTER_TEST => {
                LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(53))
            }
            _ => LpCoreWakeupSource::HpCpu,
        };
        start_ulp_core(core, ulp_wake_src, command);
    }

    // #[test]
    // fn ulp_can_start_once(ctx: Context) {
    //     _ulp_test_runner(ctx.p.ULP_RISCV_CORE, UlpCommandType::NOOP);
    //     let a = UlpLoopCounter::read();
    //     hil_test::assert_eq!(a, 1);
    // }

    // #[test]
    // fn ulp_can_be_stopped_and_resumed(ctx: Context) {
    //     // run the blocking loop
    //     _ulp_test_runner(ctx.p.ULP_RISCV_CORE, UlpCommandType::RISCV_COUNTER_TEST);
    //     // debug the core
    //     let dbg = ulp_debug::CocpuDebug::read();
    //     defmt::println!("{:?}",dbg);
    //     hil_test::assert!(ulp_is_running());
    //     ulp_riscv_timer_stop();
    //     ulp_riscv_halt();
    //     hil_test::assert!(!ulp_is_running());
    //     ulp_riscv_timer_resume();
    //     hil_test::assert!(ulp_is_running());
    // }

    // #[test]
    // fn ulp_can_be_erased(ctx: Context) {
    //     erase_ulp_core(unsafe { ctx.p.ULP_RISCV_CORE.clone_unchecked() });
    //     hil_test::assert!(!ulp_is_running());
    //     _ulp_test_runner(ctx.p.ULP_RISCV_CORE, UlpCommandType::RISCV_NO_COMMAND);
    //     let a = UlpLoopCounter::read();
    //     hil_test::assert_eq!(a, 1);
    // }

    #[test]
    fn ulp_loop_counter(ctx: Context) {
        _ulp_test_runner(ctx.p.ULP_RISCV_CORE, UlpCommandType::LOOP_COUNTER_TEST);
        hil_test::assert_eq!(true, ulp_is_running());
        hil_test::assert_eq!(UlpReply::read(), UlpReplyType::REPLY_OK);
    }

    // #[test]
    // fn ulp_timer_counter(ctx: Context) {
    //     _ulp_test_runner(
    //         ctx.p.ULP_RISCV_CORE,
    //         UlpCommandType::RISCV_ULP_TIMER_COUNTER_TEST,
    //     );
    //     // print debug info for the ulp core
    //     let dbg = ulp_debug::CocpuDebug::read();
    //     defmt::println!("{:?}", dbg);

    //     match dbg.decode_instruction() {
    //         Ok(i) => {
    //             defmt::println!("{:?}", defmt::Debug2Format(&i));
    //         }
    //         Err(e) => {
    //             defmt::println!("{:?}", defmt::Debug2Format(&e));
    //         }
    //     }
    //     hil_test::assert!(ulp_is_running());
    //     hil_test::assert_eq!(UlpReply::read(), UlpReplyType::RISCV_COMMAND_OK);
    // }

    #[test]
    fn creating_peripheral_does_not_break_debug_connection(ctx: Context) {
        use esp_hal::usb_serial_jtag::UsbSerialJtag;
        _ = UsbSerialJtag::new(ctx.p.USB_DEVICE).into_async().split();
    }
}
