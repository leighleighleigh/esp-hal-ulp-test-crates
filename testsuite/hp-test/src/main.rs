#![no_std]
#![no_main]
#[allow(unused_imports)]
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{DriveMode, Flex, OutputConfig, Pull, RtcPin, RtcPinWithResistors},
    peripherals::GPIO2,
    // load_lp_code,
    main,
    time::Instant,
    ulp_core::{
        // UlpCore as LpCore,
        UlpCoreTimerCycles as LpCoreTimerCycles,
        UlpCoreWakeupSource as LpCoreWakeupSource,
    },
};
use hil_test::{
    ulp_debug::{CocpuDebug, FromRegister},
    ulp_utils,
};
use shared::{UlpCommandType, UlpLoopCounter};

// Type aliasing for peripheral type
type LpCorePeripheral = esp_hal::peripherals::ULP_RISCV_CORE<'static>;

// Setup the ULP core
fn setup_ulp_core(core: LpCorePeripheral) {
    // ulp_utils::ulp_riscv_reset();
    ulp_utils::start_ulp_core(
        core,
        LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(53)),
        // LpCoreWakeupSource::HpCpu,
        UlpCommandType::LOOP_COUNTER_TEST,
    );
}

#[main]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    {
        // REQUIRED FOR LEIGHLEIGHLEIGH's CUSTOM DEVBOARD ONLY
        // Turn the power on, and keep it on during sleep using pad hold.
        let mut io_reg_en = peripherals.GPIO2;
        let mut reg_enable = Flex::new(io_reg_en.reborrow());
        reg_enable.apply_output_config(
            &OutputConfig::default()
                .with_drive_mode(DriveMode::OpenDrain)
                .with_pull(Pull::Up),
        );
        reg_enable.set_high();
        <GPIO2 as RtcPin>::rtcio_pad_hold(&io_reg_en, true);
        <GPIO2 as RtcPinWithResistors>::rtcio_pullup(&io_reg_en, true);
    }

    // Delay to allow USB to connect
    let dly = Delay::new();
    dly.delay_millis(500);

    log::info!("HP core started! Counter: {}", UlpLoopCounter::read());

    // re-program the ULP everytime HP core boots
    setup_ulp_core(peripherals.ULP_RISCV_CORE);
    dly.delay_millis(500);

    log::info!("LP core started! Counter: {}", UlpLoopCounter::read());

    // Sample the counter in a loop
    let mut count = UlpLoopCounter::read();
    let mut timestamp = Instant::now();
    let mut debug_timestamp = Instant::now();

    loop {
        let new_count = UlpLoopCounter::read();
        let new_time = Instant::now();

        if count != new_count {
            let time_delta = new_time - timestamp;
            log::info!("[{} us] counter: {} -> {}", time_delta.as_micros(), count, new_count);
            count = new_count;
            timestamp = new_time;
            debug_timestamp = timestamp;
        }

        if debug_timestamp.elapsed().as_secs() >= 2 {
            // print debug info for the ulp core
            let dbg = CocpuDebug::read();
            log::info!("{:?}", dbg);

            match dbg.decode_instruction() {
                Ok(i) => {
                    log::info!("{:?}", i);
                }
                Err(e) => {
                    log::info!("{:?}", e);
                }
            }

            debug_timestamp = Instant::now();
        }
    }
}
