#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use log::info;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::main;

#[allow(unused_imports)]
use esp_hal::time::{Duration, Instant};
use esp_hal::delay::Delay;

use esp_hal::load_lp_code;

#[cfg(any(esp32s2,esp32s3))]
use esp_hal::ulp_core::{UlpCore,UlpCoreWakeupSource};
#[cfg(feature = "ulp-stompy")]
use esp_hal::gpio::rtc_io::{LowPowerOutput,LowPowerInput,LowPowerOutputOpenDrain};

#[cfg(any(esp32s2,esp32s3))]
mod ulp_debug;

#[cfg(esp32c6)]
use esp_hal::lp_core::{LpCore, LpCoreWakeupSource};

// For power pin
use esp_hal::peripherals::GPIO2;
use esp_hal::gpio::{Flex,DriveMode,Pull,OutputConfig,RtcPin,RtcPinWithResistors};

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

    // First thing to do is keep the power on, and keep it on during sleep using pad hold.
    // 3V3_EN_B - IO2
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


    #[cfg(any(esp32s2,esp32s3))]
    let mut ulp_core = UlpCore::new(peripherals.ULP_RISCV_CORE).with_sleep_cycles(53); // 53 cycles is about 10Hz counter increment (timer loop would be slightly faster)

    #[cfg(esp32c6)]
    let mut ulp_core = LpCore::new(peripherals.LP_CORE);

    #[cfg(feature = "ulp-stompy")]
    {
        // Install ulp-apps/esp32s3-stompy-ulp-lp-core-ws2812b
        #[cfg(esp32s3)]
        let ulp_core_code = load_lp_code!("./ulp-apps/esp32s3-ulp-stompy");

        let ulp_arg_pin0 = LowPowerOutputOpenDrain::new(peripherals.GPIO0);
        let ulp_arg_pin1 = LowPowerOutputOpenDrain::new(peripherals.GPIO1);
        let ulp_arg_pin = LowPowerInput::new(peripherals.GPIO5);
        let ulp_arg_out_io18 = LowPowerOutput::new(peripherals.GPIO18);
        // ulp_core.stop();

        ulp_core_code.run(
            &mut ulp_core,
            UlpCoreWakeupSource::HpCpu,
            ulp_arg_pin0,
            ulp_arg_pin1,
            ulp_arg_pin,
            ulp_arg_out_io18,
        );
    }

    let counter_ptr = (0x5000_1000) as *mut u32;

    #[cfg(feature = "ulp-blinky")]
    {
        // #[cfg(esp32s3)]
        // ulp_core.stop();

        // Reset the counter
        unsafe { counter_ptr.write_volatile(0); }
        
        // Install ulp-apps/$CHIP-blinky
        #[cfg(esp32s3)]
        let ulp_core_code = load_lp_code!("./ulp-apps/esp32s3-ulp-blinky");

        #[cfg(esp32s2)]
        let ulp_core_code = load_lp_code!("./ulp-apps/esp32s2-ulp-blinky");

        #[cfg(esp32c6)]
        let ulp_core_code = load_lp_code!("./ulp-apps/esp32c6-ulp-blinky");

        #[cfg(any(esp32s2,esp32s3))]
        ulp_core_code.run(&mut ulp_core, UlpCoreWakeupSource::HpCpu);

        #[cfg(esp32c6)]
        ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::HpCpu);
    }

    // Measure the ULP counter quickly in a loop,
    // and try to estimate the frequency of ULP counter updates.
    let _dly = Delay::new();
    let mut last_change_time = Instant::now();
    let mut last_counter = unsafe { counter_ptr.read_volatile() };
    let mut first = true;

    // Keep adding up the delta counts and delta times,
    // and make a rolling average of the rate.
    let mut rolling_counts : u32 = 0;
    let mut rolling_deltas : u32 = 0;

    loop {
        let new_count= unsafe { counter_ptr.read_volatile() };
        let new_time = Instant::now();

        if new_count != last_counter {
            if !first {
                let dc = new_count - last_counter;
                let dt = new_time - last_change_time;

                rolling_counts += dc;
                rolling_deltas += dt.as_millis() as u32;

                // Print the updated rollling amount
                // calculate the rate of single counts, in Hertz.
                // for now, just uses massive expensive floating-point operations for this, for accuracy! :)
                let count_rate : f64 = (rolling_counts as f64) / ((rolling_deltas as f64) / 1000.0);
                info!("Rolling count {}, delta {}ms, rate {}Hz", rolling_counts,rolling_deltas,count_rate);
            }
            first = false;

            // Calculate delta, calculate time delta
            last_counter = new_count;
            last_change_time = Instant::now();
        }

    }
}
