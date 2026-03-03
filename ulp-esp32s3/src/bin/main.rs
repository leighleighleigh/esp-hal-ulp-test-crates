#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::{load_lp_code,ulp_core::{UlpCore,UlpCoreWakeupSource}};
use log::info;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut ulp_core = UlpCore::new(peripherals.ULP_RISCV_CORE);
    #[cfg(not(feature = "esp32s2"))]
    {
        ulp_core.stop(); // currently not implemented for ESP32-S2.
        info!("ulp core stopped");
    }

    // load code to LP core
    let ulp_core_code = load_lp_code!(
        "../../blinky/target/riscv32imc-unknown-none-elf/release/blinky"
    );

    // start ULP coprocessor
    ulp_core_code.run(&mut ulp_core, UlpCoreWakeupSource::HpCpu);

    let data = (0x5000_0400) as *mut u32;

    loop {
        info!("ULP counter {:x}           \u{000d}", unsafe {data.read_volatile()});
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }
}
