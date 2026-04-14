#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
#[cfg(feature = "main-core-sleeps")]
use esp_hal::rtc_cntl::sleep::{RtcSleepConfig, UlpWakeupSource, WakeSource};
#[allow(unused_imports)]
use esp_hal::time::{Duration, Instant};
#[cfg(any(esp32s2, esp32s3))]
use esp_hal::ulp_core::{UlpCore, UlpCoreTimerCycles, UlpCoreWakeupSource};
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{InputConfig, WakeEvent},
    load_lp_code,
    main,
    peripherals::RTC_IO,
    system::{SleepSource, wakeup_cause},
};
use log::{error, info};

#[cfg(any(esp32s2, esp32s3))]
mod ulp_debug;

use esp_hal::gpio::{
    DriveMode,
    Flex,
    OutputConfig,
    Pull,
    InputPin,
    RtcPin,
    RtcPinWithResistors,
    rtc_io::LowPowerInput,
};
#[cfg(esp32c6)]
use esp_hal::lp_core::{LpCore, LpCoreWakeupSource};
// For power pin
use esp_hal::peripherals::{GPIO5, GPIO0, GPIO2};

use crate::ulp_debug::FromRegister;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Timeout period when no count is detected
const SAMPLE_TIMEOUT_MILLIS: u64 = 2500;

// Affects how fast the ULP code is executed
const ULP_SLEEP_CYCLES: u32 = 53;

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Set GPIO2 as an output, and set its state high initially.
    // let mut io = esp_hal::gpio::Io::new(peripherals.IO_MUX);
    // io.set_interrupt_handler(handler);

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
            #[cfg(any(esp32s2, esp32s3))]
            let mut ulp_core = UlpCore::new(peripherals.ULP_RISCV_CORE);
            #[cfg(esp32c6)]
            let mut ulp_core = LpCore::new(peripherals.LP_CORE);

            // Load the application
            #[cfg(esp32s3)]
            let ulp_core_code = load_lp_code!("./ulp-apps/esp32s3-ulp-blinky");
            #[cfg(esp32s2)]
            let ulp_core_code = load_lp_code!("./ulp-apps/esp32s2-ulp-blinky");
            #[cfg(esp32c6)]
            let ulp_core_code = load_lp_code!("./ulp-apps/esp32c6-ulp-blinky");


            // STOMP PIN
            let ulp_button = unsafe { peripherals.GPIO5.clone_unchecked() };
            let ulp_arg_pin = LowPowerInput::new(ulp_button);
            ulp_arg_pin.wakeup_enable(Some(WakeEvent::HighLevel));
            
            // Needed for sleep.
            // unsafe {
            //    // Hold the GPIO pin, and also the RTC pin, settings.
            //    let ulp_button_rtc = unsafe { peripherals.GPIO5.clone_unchecked() };
            //    <GPIO5 as RtcPin>::rtcio_pad_hold(&ulp_button_rtc, true);
            // }

            /* Configure the button GPIO as input, enable wakeup */
            // rtc_gpio_init(WAKEUP_PIN);
            // rtc_gpio_set_direction(WAKEUP_PIN, RTC_GPIO_MODE_INPUT_ONLY);
            // rtc_gpio_pulldown_dis(WAKEUP_PIN);
            // rtc_gpio_pullup_en(WAKEUP_PIN);
            // rtc_gpio_wakeup_enable(WAKEUP_PIN, GPIO_INTR_NEGEDGE);
            
            // Reset the counter
            unsafe {
                counter_ptr.write_volatile(0);
            }

            #[cfg(any(esp32s2, esp32s3))]
            ulp_core_code.run(
                &mut ulp_core,
                // UlpCoreWakeupSource::Timer(UlpCoreTimerCycles::new(ULP_SLEEP_CYCLES)),
                UlpCoreWakeupSource::Gpio,
                ulp_arg_pin,
            );

            #[cfg(esp32c6)]
            ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::HpCpu);
        }
    }

    // Measure the ULP counter quickly in a loop,
    // and try to estimate the frequency of ULP counter updates.
    let mut waiting_first_sample = true;
    let mut last_change_time = Instant::now();
    let mut last_counter = unsafe { counter_ptr.read_volatile() };

    let mut single_count_samples: u64 = 0;
    let mut single_count_period: u64 = 0;

    loop {
        let new_count = unsafe { counter_ptr.read_volatile() };
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
            info!(
                "value {new_count}, samples {single_count_samples}, avg_period {avg_period}, avg_rate {avg_rate}"
            );

            if waiting_first_sample {
                waiting_first_sample = false;
                single_count_samples = 0;
                single_count_period = 0;
            }
        } else if last_change_time.elapsed().as_millis() > SAMPLE_TIMEOUT_MILLIS {
            info!("No change in {SAMPLE_TIMEOUT_MILLIS}... (value: {new_count})");
            last_counter = new_count;
            last_change_time = new_time;
            waiting_first_sample = true;

            // Print some debug information
            let cocpu_debug = ulp_debug::CocpuDebug::read();
            info!("{cocpu_debug:?}");
            ulp_debug::dump_coproc_pc_instructions(&cocpu_debug);
            let (_pc, instr) = ulp_debug::get_cocpu_pc_instr(&cocpu_debug);
            // Decode the instruction type
            match riscv_decode::decode(instr) {
                Ok(i) => {
                    info!("{i:?}");
                }
                Err(e) => {
                    error!("{e:?}");
                }
            };

            #[cfg(feature = "main-core-sleeps")]
            {
              break;
            }
        }
    }

    // Now go to sleep deep sleep, until the ulp wakes us up!
    #[cfg(feature = "main-core-sleeps")]
    {
        let mut rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);
        let ulp_wakeup = UlpWakeupSource::new();
        let wake_sources: &[&dyn WakeSource] = &[&ulp_wakeup];

        let config: RtcSleepConfig = RtcSleepConfig::deep();
        rtc.sleep(&config, &wake_sources);

        loop {}
    }
}
