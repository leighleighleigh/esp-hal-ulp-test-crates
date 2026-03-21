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
use esp_hal::system::{SleepSource,wakeup_cause};
use esp_hal::rtc_cntl::sleep::{UlpWakeupSource,RtcSleepConfig,WakeSource};

#[allow(unused_imports)]
use esp_hal::time::{Duration, Instant};
use esp_hal::delay::Delay;

use esp_hal::load_lp_code;

#[cfg(any(esp32s2,esp32s3))]
use esp_hal::ulp_core::{UlpCore,UlpCoreWakeupSource,UlpCoreTimerCycles};

#[cfg(any(esp32s2,esp32s3))]
mod ulp_debug;

#[cfg(esp32c6)]
use esp_hal::lp_core::{LpCore, LpCoreWakeupSource};

// For power pin
use esp_hal::peripherals::GPIO2;
use esp_hal::gpio::{Flex,DriveMode,Pull,OutputConfig,RtcPin,RtcPinWithResistors};

use esp_hal::gpio::rtc_io::LowPowerInput;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();


const ULP_SLEEP_CYCLES : u32 = 53; // Affects how fast the ULP code is executed
const SAMPLE_LOOP_COUNT : u32 = 10;


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

    // Pointer to the shared counter variable in memory
    let counter_ptr = (0x5000_1000) as *mut u32;

    // Check ESP wake-up condition.
    let wakeup_reason = wakeup_cause();

    match wakeup_reason {
        SleepSource::Ulp => {
            info!("Woke from ULP interrupt!");
        }
        _ => {
            // Else, reprogram the ULP
            #[cfg(any(esp32s2,esp32s3))]
            let mut ulp_core = UlpCore::new(peripherals.ULP_RISCV_CORE);
            #[cfg(esp32c6)]
            let mut ulp_core = LpCore::new(peripherals.LP_CORE);

            let ulp_arg_pin = LowPowerInput::new(peripherals.GPIO5);

            // Load the application
            #[cfg(esp32s3)]
            let ulp_core_code = load_lp_code!("./ulp-apps/esp32s3-ulp-blinky");
            #[cfg(esp32s2)]
            let ulp_core_code = load_lp_code!("./ulp-apps/esp32s2-ulp-blinky");
            #[cfg(esp32c6)]
            let ulp_core_code = load_lp_code!("./ulp-apps/esp32c6-ulp-blinky");

            // Reset the counter
            unsafe { counter_ptr.write_volatile(0); }

            #[cfg(any(esp32s2,esp32s3))]
            ulp_core_code.run(&mut ulp_core, UlpCoreWakeupSource::Timer(UlpCoreTimerCycles::new(ULP_SLEEP_CYCLES)), ulp_arg_pin);

            #[cfg(esp32c6)]
            ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::HpCpu);
        }
    }

    // Delay to allow USB to connect
    let dly = Delay::new();
    dly.delay_millis(500);

    // Measure the ULP counter quickly in a loop,
    // and try to estimate the frequency of ULP counter updates.
    let mut loop_limit = SAMPLE_LOOP_COUNT;

    let mut last_change_time = Instant::now();
    let mut last_counter = unsafe { counter_ptr.read_volatile() };

    let mut single_count_samples : u64 = 0;
    let mut single_count_period : u64 = 0;

    loop {
        let new_count= unsafe { counter_ptr.read_volatile() };
        let new_time = Instant::now();

        if new_count != last_counter {
            let dc = new_count - last_counter;
            let dt = new_time - last_change_time;

            // Calculate micros
            let dtmicros = dt.as_micros();
            // calculate micros per count
            let count_period = dtmicros / (dc as u64);

            single_count_samples += 1;
            single_count_period += count_period;
            last_counter = new_count;
            last_change_time = new_time;

            let avg_period = single_count_period / single_count_samples;
            let avg_rate = 1000000.0 / (avg_period as f64);
            info!("value {}, samples {}, avg_period {}, avg_rate {}",new_count,single_count_samples,avg_period,avg_rate);

            #[cfg(feature = "main-core-sleeps")]
            {
                loop_limit -= 1;
            }

            if loop_limit == 0 {
                break;
            }
        } else {
            if last_change_time.elapsed().as_secs() > 5 {
                info!("No change in 5 seconds... (value: {})",new_count);
                last_counter = new_count;
                last_change_time = new_time;
            }
        }
    }

    // Now go to sleep deep sleep, until the ulp wakes us up!
    let mut rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);
    let ulp_wakeup = UlpWakeupSource::new();
    let wake_sources: &[&dyn WakeSource] = &[&ulp_wakeup];
    let config : RtcSleepConfig = RtcSleepConfig::deep();
    rtc.sleep(&config, &wake_sources);

    loop {}
}
