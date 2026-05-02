#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::time;

use esp_backtrace as _;
#[cfg(esp32c6)]
use esp_hal::gpio::lp_io::LowPowerInput;
#[cfg(esp32c6)]
use esp_hal::lp_core::{LpCore, LpCoreWakeupSource};
#[allow(unused_imports)]
use esp_hal::time::{Duration, Instant};
#[cfg(any(esp32s2, esp32s3))]
use esp_hal::ulp_core::{
    UlpCore as LpCore,
    UlpCoreWakeupSource as LpCoreWakeupSource,
};

#[cfg(feature = "use-ulp-timer")]
use esp_hal::ulp_core::UlpCoreTimerCycles as LpCoreTimerCycles;

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{
        DriveMode,
        Flex,
        OutputConfig,
        Pull,
        RtcPin,
        RtcPinWithResistors,
    },
    load_lp_code,
    main,
    peripherals::GPIO2
};
use log::info;

mod counter;
use counter::{COUNTER_ADDRESS, MODE_ADDRESS, reg_read, reg_write};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Type aliasing for peripheral type
#[cfg(any(esp32s2, esp32s3))]
type LpCorePeripheral = esp_hal::peripherals::ULP_RISCV_CORE<'static>;
#[cfg(esp32c6)]
type LpCorePeripheral = esp_hal::peripherals::LP_CORE<'static>;

// Setup the ULP core
fn setup_ulp_core(core: LpCorePeripheral) {
    // Else, reprogram the ULP
    let mut ulp_core = LpCore::new(core);

    // Load the application
    #[cfg(esp32s3)]
    let ulp_core_code = load_lp_code!("lp_app");
    #[cfg(esp32s2)]
    let ulp_core_code = load_lp_code!("lp_app");
    #[cfg(esp32c6)]
    let ulp_core_code = load_lp_code!("lp_app");

    // Reset the counter & mode
    reg_write(COUNTER_ADDRESS, 0);

    #[cfg(not(feature = "use-ulp-timer"))]
    {
        info!("LP core will be woken from HP Cpu");
        reg_write(MODE_ADDRESS, 0);
    }
    #[cfg(feature = "use-ulp-timer")]
    {
        info!("LP core will be woken from ULP Timer");
        reg_write(MODE_ADDRESS, 1);
    }

    #[cfg(any(esp32s2, esp32s3))]
    ulp_core_code.run(
        &mut ulp_core,
        #[cfg(not(feature = "use-ulp-timer"))]
        LpCoreWakeupSource::HpCpu,
        #[cfg(feature = "use-ulp-timer")]
        LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(530)) // approx 1 Hz
    );

    #[cfg(esp32c6)]
    ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::HpCpu);
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

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

    // re-program the ULP everytime HP core boots
    #[cfg(esp32c6)]
    setup_ulp_core(peripherals.LP_CORE);
    #[cfg(any(esp32s2, esp32s3))]
    setup_ulp_core(peripherals.ULP_RISCV_CORE);

    // Sample the counter in a loop
    let mut count = 0;
    let mut timestamp = Instant::now();

    loop {
        let new_count = reg_read(COUNTER_ADDRESS);
        let new_time = Instant::now();

        if count != new_count {
            let time_delta = new_time - timestamp;
            info!("[{:+} us] counter: {}", time_delta.as_micros(), count);
            count = new_count;
            timestamp = new_time;
        }
    }
}
