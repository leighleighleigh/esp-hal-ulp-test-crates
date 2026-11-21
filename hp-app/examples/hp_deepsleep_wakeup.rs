#![no_std]
#![no_main]
#![allow(unused_imports)]
#![allow(static_mut_refs)]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{DriveMode, Flex, OutputConfig, Pull, lp_io::LowPowerPin},
    interrupt,
    load_lp_code,
    lp_core::{
        UlpCore as LpCore,
        UlpCoreTimerCycles as LpCoreTimerCycles,
        UlpCoreWakeupSource as LpCoreWakeupSource,
        WakeupConfig as LpWakeupConfig,
    },
    main,
    peripherals::{self, GPIO2, Peripherals},
    rtc_cntl::{
        SocResetReason,
        WakeupReason,
        WakeupSource,
        reset_reason,
        sleep::{LowPower, RtcSleepConfig},
        wakeup_cause,
    },
    time::Instant,
};
use esp32s3 as pac;
use hil_test::ulp_utils::ulp_riscv_reset;
// Shared variables from lp_rainbow app
use lp_rainbow::{RAINBOW_COUNTER, RAINBOW_RUN};
// Global on/off controls
const ULP_CORE_ENABLED: bool = true;
const ULP_RAINBOW_DURING_SLEEP: bool = true;
const ULP_RAINBOW_SPEED: u32 = 10; // Fast

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    {
        // REQUIRED FOR LEIGHLEIGHLEIGH's CUSTOM DEVBOARD ONLY
        let io_reg_en = unsafe { p.GPIO2.clone_unchecked() };
        let chg_reset = unsafe { p.GPIO7.clone_unchecked() };
        setup_gpio_holds(io_reg_en, chg_reset);
    }

    defmt::info!("Boot hp_deepsleep_wakeup!");

    // Pause the rainbow, we are awake!
    unsafe {
        RAINBOW_RUN = 0;
    }

    // Figure out the wake reason, if it was due to Ulp, don't re-program.
    let mut ulp_was_running: bool = false;
    let wake = esp_hal::system::wakeup_cause();
    for reason in wake.iter() {
        if reason == WakeupSource::UlpRiscv {
            ulp_was_running = true;
        }
    }

    let mut lpwr = LowPower::new(p.LPWR);
    let mut ulp_core = LpCore::new(p.ULP_RISCV_CORE);

    if ULP_CORE_ENABLED {
        ulp_core.enable_wakeup(LpWakeupConfig::default()); // HP can be woken by ULP

        if !ulp_was_running {
            // Chill for a couple seconds, allowing serial port to connect
            let dly = esp_hal::delay::Delay::new();
            dly.delay_millis(2000);

            defmt::warn!("Re-programming ULP!");
            ulp_core.erase();
            let ulp_code = esp_hal::load_lp_code!(
                "../lp-ws2812-app/target/riscv32imc-unknown-none-elf/release/lp_rainbow"
            );

            let lp_led_pin = esp_hal::gpio::Output::new(
                p.GPIO18,
                esp_hal::gpio::Level::Low,
                esp_hal::gpio::OutputConfig::default()
                    .with_drive_mode(esp_hal::gpio::DriveMode::PushPull)
                    .with_pull(esp_hal::gpio::Pull::Down),
            )
            .into_lp()
            .expect("got LP pin");

            let lp_button_pin = esp_hal::gpio::Input::new(
                p.GPIO5,
                esp_hal::gpio::InputConfig::default().with_pull(esp_hal::gpio::Pull::None),
            )
            .into_lp()
            .unwrap();

            // Only when .run is called, will the ULP core be re-flashed.
            let wakeup_source =
                LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(ULP_RAINBOW_SPEED));

            ulp_code.run(&mut ulp_core, wakeup_source, lp_led_pin, lp_button_pin);

            defmt::warn!("ULP reprogrammed!");
        }
    }

    let count = unsafe { RAINBOW_COUNTER };
    defmt::info!("RAINBOW_COUNTER = {}", count);
    defmt::info!("Resetting RAINBOW_COUNTER");

    // Reset counter to 0
    unsafe {
        RAINBOW_COUNTER = 0;
    }

    defmt::warn!("Entering deep sleep!");

    if ULP_RAINBOW_DURING_SLEEP {
        unsafe {
            RAINBOW_RUN = 0xCAFEBABE;
        }
    }

    if !ULP_CORE_ENABLED {
        let wakeup_deadline = esp_hal::time::Duration::from_millis(10000);
        lpwr.set_wakeup_deadline(Instant::now() + wakeup_deadline);
    }

    let mut sleep_cfg = RtcSleepConfig::default();
    if ULP_RAINBOW_DURING_SLEEP {
        sleep_cfg.set_rtc_peri_pd_en(false); // keep power on for low-power pins
    }

    lpwr.sleep_deep(sleep_cfg);
}

fn setup_gpio_holds(mut io_reg_en: peripherals::GPIO2, mut chg_reset: peripherals::GPIO7) {
    // REQUIRED FOR LEIGHLEIGHLEIGH's CUSTOM DEVBOARD ONLY
    // Turn the power on, and keep it on during sleep using pad hold.
    let mut reg_enable = esp_hal::gpio::Flex::new(io_reg_en.reborrow());
    reg_enable.apply_output_config(
        &esp_hal::gpio::OutputConfig::default()
            .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
            .with_pull(esp_hal::gpio::Pull::Up),
    );
    reg_enable.set_high();
    reg_enable.set_pad_hold(true);

    // Turn off the charger
    let mut chg_enable = esp_hal::gpio::Flex::new(chg_reset.reborrow());
    chg_enable.apply_output_config(
        &esp_hal::gpio::OutputConfig::default()
            .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
            .with_pull(esp_hal::gpio::Pull::None),
    );
    chg_enable.set_high();
    let dly = esp_hal::delay::Delay::new();
    dly.delay_millis(1);
    chg_enable.set_low();
    dly.delay_millis(1);
    chg_enable.set_high();
    chg_enable.set_pad_hold(true);
}
