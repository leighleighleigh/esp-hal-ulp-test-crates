#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal as _;
use esp_hal::{clock::CpuClock, main};
use hil_test::{info, setup_logger};

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);
    setup_logger();
    info!("Boot!");

    {
        // REQUIRED FOR LEIGHLEIGHLEIGH's CUSTOM DEVBOARD ONLY
        // Turn the power on, and keep it on during sleep using pad hold.
        let mut io_reg_en = unsafe { p.GPIO2.clone_unchecked() };
        let mut reg_enable = esp_hal::gpio::Flex::new(io_reg_en.reborrow());
        reg_enable.apply_output_config(
            &esp_hal::gpio::OutputConfig::default()
                .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                .with_pull(esp_hal::gpio::Pull::Up),
        );
        reg_enable.set_high();
        reg_enable.set_pad_hold(true);
    }

    let dly = esp_hal::delay::Delay::new();
    let mut loop_count = 0;
    loop {
        info!("Loop {}", loop_count);
        loop_count += 1;
        dly.delay_millis(1000);
    }
}
