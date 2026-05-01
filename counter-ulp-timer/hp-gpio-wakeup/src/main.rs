#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::any::Any;

use esp_backtrace as _;

#[cfg(any(esp32s2,esp32s3))]
use esp_hal::{peripherals::{self, RTC_CNTL, SENS}, rtc_cntl::sleep::{RtcSleepConfig, WakeFromUlpCoreWakeupSource, WakeSource}};

#[cfg(esp32c6)]
use esp_hal::{peripherals, rtc_cntl::sleep::{RtcSleepConfig, WakeFromLpCoreWakeupSource, WakeSource}};

#[allow(unused_imports)]
use esp_hal::time::{Duration, Instant};

#[cfg(any(esp32s2, esp32s3))]
use esp_hal::ulp_core::{UlpCore, UlpCoreTimerCycles, UlpCoreWakeupSource};

#[cfg(esp32c6)]
use esp_hal::lp_core::{LpCore, LpCoreClockSource, LpCoreWakeupSource};

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::WakeEvent,
    load_lp_code,
    main,
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
};

#[cfg(any(esp32s2, esp32s3))]
use esp_hal::gpio::rtc_io::LowPowerInput;

#[cfg(esp32c6)]
use esp_hal::gpio::lp_io::LowPowerInput;

// For power pin
#[allow(unused_imports)]
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

    let mut rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);
    let start = Instant::now();
    rtc.set_current_time_us(start.duration_since_epoch().as_micros());

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
    #[cfg(any(esp32s2,esp32s3))]
    let wakeup_button = peripherals.GPIO5;
    #[cfg(esp32c6)]
    let wakeup_button = peripherals.GPIO0; // Unfortunately on the XIAO C6, the BOOT button is IO9, which is not LP-capable.

    // Create the input and enable wakeup on it
    let ulp_input_argument = LowPowerInput::new(unsafe { wakeup_button.clone_unchecked() });
    
    // Enable wakeup when the button is pressed
    #[cfg(esp32c6)]
    ulp_input_argument.pulldown_enable(true);

    ulp_input_argument.wakeup_enable(Some(WakeEvent::HighLevel));

    // Enable pad hold on the wakeup button, so it's configuration 
    // does not change during sleep.
    let rtc_wakeup_button : &dyn RtcPinWithResistors = &wakeup_button;
    rtc_wakeup_button.rtcio_pad_hold(true);
    rtc_wakeup_button.rtcio_pulldown(true);
    //unsafe { rtc_wakeup_button.apply_wakeup(true, WakeEvent::HighLevel as u8) };

    // For ESP32C6, also need to enable the LpIO clock
    // #[cfg(esp32c6)]
    // {
    //     let sens = peripherals::LP_PERI::regs();
    //     sens.clk_en().write(|w| w.lp_io_ck_en().set_bit());
    // }

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
            let ulp_core_code = load_lp_code!("./lp_app");
            #[cfg(esp32s2)]
            let ulp_core_code = load_lp_code!("./lp_app");
            #[cfg(esp32c6)]
            let ulp_core_code = load_lp_code!("./lp_app");

            
            #[cfg(any(esp32s2, esp32s3))]
            ulp_core_code.run(
                &mut ulp_core,
                UlpCoreWakeupSource::Gpio,
                ulp_input_argument,
            );

            #[cfg(esp32c6)]
            ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::HpCpu, ulp_input_argument);
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

    #[cfg(any(esp32s2, esp32s3))]
    let ulp_wakeup = WakeFromUlpCoreWakeupSource::new();
    #[cfg(esp32c6)]
    let ulp_wakeup = WakeFromLpCoreWakeupSource::new();

    let wake_sources: &[&dyn WakeSource] = &[&ulp_wakeup];
    let config: RtcSleepConfig = RtcSleepConfig::deep();
    
    // SAME AS RtcPin method
    // let lpwr = peripherals::LPWR::regs();
    // lpwr.pad_hold().modify(|_, w| w.touch_pad5().bit(true));

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
