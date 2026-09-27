#![no_std]
#![no_main]
#![allow(unused)]

use esp32s3 as pac;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    lp_core::{
        UlpCore as LpCore,
        UlpCoreTimerCycles as LpCoreTimerCycles,
        UlpCoreWakeupSource as LpCoreWakeupSource,
        WakeupConfig as LpWakeupConfig,
    },
    main,
    rtc_cntl::{
        sleep::{LowPower, RtcSleepConfig},
        WakeupSource,
    },
};
use esp_println as _;
use hil_test::{hp_utils::configure_rtc_pin, ulp_utils::reprogram_ulp_core_with_run_hook};
use log::{debug, info, warn, LevelFilter};
use shared::{
    SharedType,
    UlpBootCounter,
    UlpCommand,
    ULP_DEBUG_GPIO_ISR_COUNT,
    ULP_DEBUG_GPIO_ISR_STATUS,
    ULP_DEBUG_LAST_ISR_DATA,
};

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::logger::init_logger(LevelFilter::Info);

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

    // CONFIGURE PIN INTERRUPTS AND GPIO WAKEUP SOURCE
    {
        for pin in 0..=14 {
            // disable interrupts, wakeup , on all pins
            configure_rtc_pin(pin, 0, false);
        }

        // Configure pin 5 as a high-level wakeup (interrupt 5).
        // GPIO wakeup can use high or low level signals.
        // Its assumes there is a button or something attached to pin5.
        configure_rtc_pin(5, 5, true);
    }

    // Chill for a couple seconds, allowing serial port to connect
    let dly = esp_hal::delay::Delay::new();
    dly.delay_millis(750);

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
        warn!("Re-programming ULP!");
        let lp_wake_src = LpCoreWakeupSource::Gpio;
        reprogram_ulp_core_with_run_hook(&mut ulp_core, lp_wake_src, || {
            UlpCommand::GPIO_WAKEUP_TEST.store();
        });
        warn!("ULP re-programmed!");
    }

    // Print 3 times
    for _ in 0..3 {
        let new_boot_counter = UlpBootCounter::load();
        let new_gpio_isr_count = unsafe { ULP_DEBUG_GPIO_ISR_COUNT };
        let trap_data = unsafe { ULP_DEBUG_GPIO_ISR_STATUS };
        let isr_data = unsafe { ULP_DEBUG_LAST_ISR_DATA };
        info!("({}) BOOT COUNT: {}", new_gpio_isr_count, new_boot_counter);
        info!("({}) TRAP_DATA: 0x{:08x}", new_gpio_isr_count, trap_data);
        info!("({}) ISR_DATA:  0x{:08x}", new_gpio_isr_count, isr_data);
        dly.delay_millis(1000);
    }

    // let wakeup_deadline = esp_hal::time::Duration::from_millis(10000);
    // lpwr.set_wakeup_deadline(Instant::now() + wakeup_deadline);
    let mut sleep_cfg = RtcSleepConfig::default();
    sleep_cfg.set_rtc_peri_pd_en(false); // keep power on

    // TODO: Clear any pending wake-up events requested by the ULP core,
    // else we may immediately wake up again (e.g. if the button was pressed while running the loop
    // above)
    lpwr.sleep_deep(sleep_cfg);
}
