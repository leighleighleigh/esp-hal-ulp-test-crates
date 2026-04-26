#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
#[cfg(esp32c6)]
use esp_hal::gpio::lp_io::LowPowerInput;
#[cfg(esp32c6)]
use esp_hal::lp_core::{LpCore, LpCoreWakeupSource};
#[cfg(feature = "deep-sleep")]
use esp_hal::rtc_cntl::sleep::{RtcSleepConfig, UlpWakeupSource, WakeSource};

#[cfg(any(esp32s2, esp32s3))]
use esp_hal::gpio::rtc_io::LowPowerInput;

#[allow(unused_imports)]
use esp_hal::time::{Duration, Instant};

#[cfg(any(esp32s2, esp32s3))]
use esp_hal::ulp_core::{
    UlpCore as LpCore,
    UlpCoreTimerCycles as LpCoreTimerCycles,
    UlpCoreWakeupSource as LpCoreWakeupSource,
};
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{
        DriveMode,
        Flex,
        InputConfig,
        InputPin,
        OutputConfig,
        Pin,
        Pull,
        RtcPin,
        RtcPinWithResistors,
        WakeEvent,
    },
    load_lp_code,
    main,
    peripherals::{GPIO0, GPIO2, GPIO5},
    system::{SleepSource, wakeup_cause},
};
use log::{error, info};

#[cfg(any(esp32s2, esp32s3))]
mod ulp_debug;
#[cfg(any(esp32s2, esp32s3))]
use crate::ulp_debug::FromRegister;

mod counter_tacho;
use counter_tacho::{DEBUG_ADDRESS, ADDRESS,reg_read,reg_write,SampleTachometer,Sample};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Timeout period when no count is detected
const SAMPLE_TIMEOUT_MILLIS: u64 = 2500;
// Affects how fast the ULP code is executed
const ULP_SLEEP_CYCLES: u32 = 53;

// Print some debug info
fn print_ulp_debug_info() {
    // Print some debug information
    #[cfg(not(esp32c6))]
    {
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
    }
}

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
    let ulp_core_code = load_lp_code!("./ulp-apps/esp32s3-ulp-blinky");
    #[cfg(esp32s2)]
    let ulp_core_code = load_lp_code!("./ulp-apps/esp32s2-ulp-blinky");
    #[cfg(esp32c6)]
    let ulp_core_code = load_lp_code!("./ulp-apps/esp32c6-ulp-blinky");

    // STOMP PIN
    let _ulp_button = unsafe { esp_hal::peripherals::GPIO5::steal() };
    let _ulp_button_in = esp_hal::gpio::Input::new(_ulp_button, InputConfig::default());

    let ulp_button = unsafe { esp_hal::peripherals::GPIO5::steal() };
    let ulp_arg_pin = LowPowerInput::new(ulp_button);
    // ulp_arg_pin.wakeup_enable(Some(WakeEvent::HighLevel));
    // ulp_arg_pin.wakeup_enable(Some(WakeEvent::HighLevel));

    // Needed for sleep.
    // unsafe {
    //    // Hold the GPIO pin, and also the RTC pin, settings.
    //    let ulp_button_rtc = unsafe { esp_hal::peripherals::GPIO5::steal() };
    //    <GPIO5 as RtcPin>::rtcio_pad_hold(&ulp_button_rtc, true);
    // }

    // Reset the counter
    reg_write(ADDRESS,0);

    #[cfg(any(esp32s2, esp32s3))]
    ulp_core_code.run(
        &mut ulp_core,
        LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(ULP_SLEEP_CYCLES)),
        ulp_arg_pin,
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

    // Check ESP wake-up condition.
    let wakeup_reason = wakeup_cause();

    // Wakeup reason differs depending on feature flags
    match wakeup_reason {
        #[cfg(feature = "deep-sleep")]
        SleepSource::Ulp => {
            info!("Woke from ULP interrupt!");
        }
        // re-program the ULP on any other wake-up condition
        _ => {
            print_ulp_debug_info();

            #[cfg(esp32c6)]
            setup_ulp_core(peripherals.LP_CORE);
            #[cfg(any(esp32s2,esp32s3))]
            setup_ulp_core(peripherals.ULP_RISCV_CORE);
        }
    }

    // Sample the counter
    let mut tacho = SampleTachometer::new();
    let mut dbg_sample = reg_read(DEBUG_ADDRESS);

    loop {
        // Do debug sampling
        let new_dbg_sample = reg_read(DEBUG_ADDRESS);
        if new_dbg_sample != dbg_sample {
            info!("DEBUG: 0x{:08x}",new_dbg_sample);
        }
        dbg_sample = new_dbg_sample;

        // Get a sample
        let sample = Sample::new();
        tacho.update(sample);

        // Keep sampling if unchanged
        if ! tacho.changed() {
            continue;
        }

        // Else print!
        info!("{:?}, {:?}, {:?}", tacho.count_period(), tacho.count_rate(), sample);

        #[cfg(feature = "deep-sleep")]
        break;
    }

    // Now go to sleep deep sleep, until the ulp wakes us up!
    #[cfg(feature = "deep-sleep")]
    {
        let mut rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);
        let ulp_wakeup = UlpWakeupSource::new();
        let wake_sources: &[&dyn WakeSource] = &[&ulp_wakeup];

        let config: RtcSleepConfig = RtcSleepConfig::deep();
        rtc.sleep(&config, &wake_sources);

        loop {}
    }
}
