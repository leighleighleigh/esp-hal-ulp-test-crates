#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]
#![allow(unused_imports)]

//% CHIPS: esp32c6 esp32s2 esp32s3
//% FEATURES: unstable

#[embedded_test::tests(default_timeout = 2)]
mod tests {
    use embedded_hal::delay::DelayNs;
    use esp_hal::{delay::Delay, peripherals::Peripherals, time::Instant};
    use hil_test::ulp_debug::FromRegister;
    use hil_test::{self as _, ulp_debug};
    use hil_test::ulp_utils::{
        LpCorePeripheral, LpCoreTimerCycles, LpCoreWakeupSource, erase_ulp_core, start_ulp_core, ulp_is_running, ulp_riscv_halt, ulp_riscv_reset, ulp_riscv_timer_resume, ulp_riscv_timer_stop
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

        // ulp_riscv_reset();
        // erase_ulp_core(unsafe { peripherals.ULP_RISCV_CORE.clone_unchecked() });
        // _ulp_test_runner(
        //     unsafe { peripherals.ULP_RISCV_CORE.clone_unchecked() },
        //     UlpCommandType::RISCV_NO_COMMAND,
        // );

        Context { p: peripherals }
    }

    // Halt ULP, set command, and start ULP
    fn _ulp_test_runner(core: LpCorePeripheral, command: UlpCommandType) {
        UlpCommand::write(command);

        let ulp_wake_src: LpCoreWakeupSource = match command {
            UlpCommandType::RISCV_ULP_TIMER_COUNTER_TEST => {
                LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(5))
            }
            _ => LpCoreWakeupSource::HpCpu,
        };

        start_ulp_core(core, ulp_wake_src);
    }

    #[test]
    fn ulp_can_be_stopped_and_resumed(ctx: Context) {
        // run the blocking loop
        _ulp_test_runner(ctx.p.ULP_RISCV_CORE, UlpCommandType::RISCV_COUNTER_TEST);

        // debug the core
        let dbg = ulp_debug::CocpuDebug::read();
        defmt::info!("{:?}",dbg);

        hil_test::assert!(ulp_is_running());

        ulp_riscv_timer_stop();
        ulp_riscv_halt();

        hil_test::assert!(!ulp_is_running());

        ulp_riscv_timer_resume();

        hil_test::assert!(ulp_is_running());
    }

    // #[test]
    // fn ulp_can_be_erased(ctx: Context) {
    //     erase_ulp_core(unsafe { ctx.p.ULP_RISCV_CORE.clone_unchecked() });
    //     hil_test::assert!(!ulp_is_running());
    //     _ulp_test_runner(ctx.p.ULP_RISCV_CORE, UlpCommandType::RISCV_NO_COMMAND);
    //     let a = UlpLoopCounter::read();
    //     hil_test::assert_eq!(a, 1);
    // }

    // #[test]
    // fn ulp_loop_counter(ctx: Context) {
    //     _ulp_test_runner(ctx.p.ULP_RISCV_CORE, UlpCommandType::RISCV_COUNTER_TEST);
    //     hil_test::assert!(ulp_is_running());
    //     hil_test::assert_eq!(UlpReply::read(), UlpReplyType::RISCV_COMMAND_OK);
    // }

    // #[test]
    // fn ulp_timer_counter(ctx: Context) {
    //     _ulp_test_runner(
    //         ctx.p.ULP_RISCV_CORE,
    //         UlpCommandType::RISCV_ULP_TIMER_COUNTER_TEST,
    //     );
    //     hil_test::assert!(ulp_is_running());
    //     hil_test::assert_eq!(UlpReply::read(), UlpReplyType::RISCV_COMMAND_OK);
    // }

    // #[test]
    // fn delay_ns() {
    //     let t1 = Instant::now();
    //     Delay::new().delay_ns(600_000);
    //     let t2 = Instant::now();

    //     assert!(t2 > t1);
    //     assert!(
    //         (t2 - t1).as_micros() >= 600u64,
    //         "diff: {:?}",
    //         (t2 - t1).as_micros()
    //     );
    // }

    // #[test]
    // fn can_boot() {
    //     #[repr(align(64))]
    //     struct Aligned {
    //         _data: [u8; 128],
    //     }

    //     #[used]
    //     static ALIGNED: Aligned = Aligned { _data: [0; 128] };
    // }

    // #[test]
    // fn creating_peripheral_does_not_break_debug_connection(ctx: Context) {
    //     use esp_hal::usb_serial_jtag::UsbSerialJtag;

    //     _ = UsbSerialJtag::new(ctx.p.USB_DEVICE).into_async().split();
    // }
}

// use esp_backtrace as _;
// #[cfg(esp32c6)]
// use esp_hal::gpio::lp_io::LowPowerInput;
// #[cfg(esp32c6)]
// use esp_hal::lp_core::{LpCore, LpCoreWakeupSource};
// #[allow(unused_imports)]
// use esp_hal::time::{Duration, Instant};
// #[cfg(any(esp32s2, esp32s3))]
// use esp_hal::ulp_core::{
//     UlpCore as LpCore,
//     UlpCoreWakeupSource as LpCoreWakeupSource,
// };
// use esp_hal::ulp_core::UlpCoreTimerCycles as LpCoreTimerCycles;
// use esp_hal::{
//     clock::CpuClock,
//     delay::Delay,
//     gpio::{
//         DriveMode,
//         Flex,
//         OutputConfig,
//         Pull,
//         RtcPin,
//         RtcPinWithResistors,
//     },
//     load_lp_code,
//     main,
//     peripherals::GPIO2
// };
// use log::info;

// use shared::{COUNTER_ADDRESS, MODE_ADDRESS, reg_read, reg_write};

// // This creates a default app-descriptor required by the esp-idf bootloader.
// // For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
// esp_bootloader_esp_idf::esp_app_desc!();

// // Type aliasing for peripheral type
// #[cfg(any(esp32s2, esp32s3))]
// type LpCorePeripheral = esp_hal::peripherals::ULP_RISCV_CORE<'static>;
// #[cfg(esp32c6)]
// type LpCorePeripheral = esp_hal::peripherals::LP_CORE<'static>;

// // Setup the ULP core
// fn setup_ulp_core(core: LpCorePeripheral) {
//     // Else, reprogram the ULP
//     let mut ulp_core = LpCore::new(core);

//     // Load the application
//     #[cfg(esp32s3)]
//     let ulp_core_code = load_lp_code!("lp_app");
//     #[cfg(esp32s2)]
//     let ulp_core_code = load_lp_code!("lp_app");
//     #[cfg(esp32c6)]
//     let ulp_core_code = load_lp_code!("lp_app");

//     // Reset the counter & mode
//     reg_write(COUNTER_ADDRESS, 0);
//     info!("LP core will be woken from ULP Timer");
//     reg_write(MODE_ADDRESS, 1);

//     #[cfg(any(esp32s2, esp32s3))]
//     ulp_core_code.run(
//         &mut ulp_core,
//         LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(530)) // approx 1 Hz
//     );

//     #[cfg(esp32c6)]
//     ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::HpCpu);
// }

// #[allow(
//     clippy::large_stack_frames,
//     reason = "it's not unusual to allocate larger buffers etc. in main"
// )]
// #[main]
// fn main() -> ! {
//     // esp_println::logger::init_logger_from_env();

//     let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
//     let peripherals = esp_hal::init(config);

//     {
//         // REQUIRED FOR LEIGHLEIGHLEIGH's CUSTOM DEVBOARD ONLY
//         // Turn the power on, and keep it on during sleep using pad hold.
//         let mut io_reg_en = peripherals.GPIO2;
//         let mut reg_enable = Flex::new(io_reg_en.reborrow());
//         reg_enable.apply_output_config(
//             &OutputConfig::default()
//                 .with_drive_mode(DriveMode::OpenDrain)
//                 .with_pull(Pull::Up),
//         );
//         reg_enable.set_high();
//         <GPIO2 as RtcPin>::rtcio_pad_hold(&io_reg_en, true);
//         <GPIO2 as RtcPinWithResistors>::rtcio_pullup(&io_reg_en, true);
//     }

//     // Delay to allow USB to connect
//     let dly = Delay::new();
//     dly.delay_millis(500);

//     // re-program the ULP everytime HP core boots
//     #[cfg(esp32c6)]
//     setup_ulp_core(peripherals.LP_CORE);
//     #[cfg(any(esp32s2, esp32s3))]
//     setup_ulp_core(peripherals.ULP_RISCV_CORE);

//     // Sample the counter in a loop
//     let mut count = 0;
//     let mut timestamp = Instant::now();

//     loop {
//         let new_count = reg_read(COUNTER_ADDRESS);
//         let new_time = Instant::now();

//         if count != new_count {
//             let time_delta = new_time - timestamp;
//             info!("[{:+} us] counter: {}", time_delta.as_micros(), count);
//             count = new_count;
//             timestamp = new_time;
//         }
//     }
// }
