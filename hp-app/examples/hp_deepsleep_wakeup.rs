#![no_std]
#![no_main]
#![allow(unused_imports)]
#![allow(static_mut_refs)]

use esp32s3 as pac;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{lp_io::LowPowerPin, DriveMode, Flex, OutputConfig, Pull},
    interrupt,
    load_lp_code,
    lp_core::{
        UlpCore as LpCore,
        UlpCoreTimerCycles as LpCoreTimerCycles,
        UlpCoreWakeupSource as LpCoreWakeupSource,
        WakeupConfig as LpWakeupConfig,
    },
    main,
    peripherals::{self, Peripherals, GPIO2},
    rtc_cntl::{
        reset_reason,
        sleep::{LowPower, RtcSleepConfig},
        wakeup_cause,
        SocResetReason,
        WakeupReason,
        WakeupSource,
    },
    time::Instant,
    xtensa_lx::interrupt::set,
};
use esp_println as _;
use hil_test::ulp_utils::ulp_riscv_reset;

unsafe extern "Rust" {
    static mut RAINBOW_PAUSE: bool;
    static mut RAINBOW_COUNTER: u32;
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    {
        // REQUIRED FOR LEIGHLEIGHLEIGH's CUSTOM DEVBOARD ONLY
        // Turn the power on, and keep it on during sleep using pad hold.
        let mut io_reg_en = unsafe { peripherals.GPIO2.clone_unchecked() };
        let mut reg_enable = esp_hal::gpio::Flex::new(io_reg_en.reborrow());
        reg_enable.apply_output_config(
            &esp_hal::gpio::OutputConfig::default()
                .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                .with_pull(esp_hal::gpio::Pull::Up),
        );
        reg_enable.set_high();
        reg_enable.set_pad_hold(true);
    }

    // Pause the rainbow, we are awake!
    unsafe {
        RAINBOW_PAUSE = true;
    }

    // Figure out the wake reason, if it was due to Ulp, don't re-program.
    let mut ulp_was_running: bool = false;
    let wake = esp_hal::system::wakeup_cause();
    for reason in wake.iter() {
        if reason == WakeupSource::UlpRiscv {
            ulp_was_running = true;
        }
    }

    let mut lpwr = LowPower::new(peripherals.LPWR);
    let mut ulp_core = LpCore::new(peripherals.ULP_RISCV_CORE);
    ulp_core.enable_wakeup(LpWakeupConfig::default()); // HP can be woken by ULP

    if !ulp_was_running {
        defmt::warn!("Re-programming ULP!");
        let ulp_code = load_lp_code!("lp_rainbow");

        let mut led_pin = esp_hal::gpio::Output::new(
            peripherals.GPIO18,
            esp_hal::gpio::Level::Low,
            esp_hal::gpio::OutputConfig::default()
                .with_drive_mode(esp_hal::gpio::DriveMode::PushPull)
                .with_pull(esp_hal::gpio::Pull::None),
        );
        led_pin.set_low();
        led_pin.set_pad_hold(false);
        let lp_led_pin = led_pin.into_lp::<18>().unwrap();

        // Only when .run is called, will the ULP core be re-flashed.
        let wakeup_source = LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(10));
        ulp_code.run(&mut ulp_core, wakeup_source, lp_led_pin);
    }

    let dly = esp_hal::delay::Delay::new();

    // Pause the rainbow, we are awake!
    unsafe {
        RAINBOW_PAUSE = true;
    }

    let count = unsafe { RAINBOW_COUNTER };
    defmt::info!("RAINBOW_COUNTER = {}", count);
    defmt::info!("Resetting RAINBOW_COUNTER");

    // Reset counter to 0
    unsafe {
        RAINBOW_COUNTER = 0;
    }

    defmt::warn!("Entering deep sleep!");

    unsafe {
        RAINBOW_PAUSE = false;
    }

    // let wakeup_deadline = esp_hal::time::Duration::from_millis(10000);
    // lpwr.set_wakeup_deadline(Instant::now() + wakeup_deadline);
    let mut sleep_cfg = RtcSleepConfig::default();
    sleep_cfg.set_rtc_peri_pd_en(false); // keep power on

    lpwr.sleep_deep(sleep_cfg);
}
