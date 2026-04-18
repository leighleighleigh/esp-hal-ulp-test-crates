#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::{peripherals::{self, RTC_CNTL, SENS}, rtc_cntl::sleep::{RtcSleepConfig, UlpWakeupSource, WakeSource}};
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

    // STOMP PIN
    let ulp_button = unsafe { peripherals.GPIO5.clone_unchecked() };
    let ulp_arg_pin = LowPowerInput::new(ulp_button);
    ulp_arg_pin.wakeup_enable(Some(WakeEvent::HighLevel));
   
    // Delay to allow USB to connect
    let dly = Delay::new();
    dly.delay_millis(1000);

    // Check ESP wake-up condition.
    let wakeup_reason = wakeup_cause();

    match wakeup_reason {
        SleepSource::Ulp => {
            info!("Woke from ULP interrupt!");
        }
        other => {
            info!("Woke from {other:?}");
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

            
            #[cfg(any(esp32s2, esp32s3))]
            ulp_core_code.run(
                &mut ulp_core,
                UlpCoreWakeupSource::Gpio,
                ulp_arg_pin,
            );

            #[cfg(esp32c6)]
            ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::HpCpu);
        }
    }


    const ADDR: usize = 0x5000_1000;
    let data = (ADDR) as *const u32;

    loop {
        info!("Current {:x}           \u{000d}", unsafe {
            data.read_volatile()
        });

        dly.delay_millis(500);

        #[cfg(feature="deep-sleep")]
        break;
    }

    info!("Going to sleep...");
    let mut rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);
    let ulp_wakeup = UlpWakeupSource::new();
    let wake_sources: &[&dyn WakeSource] = &[&ulp_wakeup];
    let config: RtcSleepConfig = RtcSleepConfig::deep();

    // SAME AS RtcPin method
    let lpwr = peripherals::LPWR::regs();
    lpwr.pad_hold().modify(|_, w| w.touch_pad5().bit(true));

    // SET_PERI_REG_MASK(rtc_io_desc[rtcio_num].reg, rtc_io_desc[rtcio_num].slpie);
    // let sens = unsafe { &*SENS::PTR };
    // sens.sar_peri_clk_gate_conf().modify(|_, w| w.iomux_clk_en().set_bit());
    // Need to enable the RTC clock so the pin can be sampled during sleep
    // SENS.sar_peri_clk_gate_conf.iomux_clk_en = enable;
    // let sens = unsafe { &*SENS::PTR };
    // sens.sar_peri_clk_gate_conf().modify(|_, w| w.iomux_clk_en().set_bit());
    // Unsure if needed
    // config.set_rtc_peri_pd_en(false);
    // config.set_rtc_regulator_fpu(true);
    // Enter sleep
    rtc.sleep(&config, &wake_sources);
    loop{}
}
