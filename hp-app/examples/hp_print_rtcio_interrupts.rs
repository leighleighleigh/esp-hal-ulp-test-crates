#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    lp_core::{
        UlpCore as LpCore,
        UlpCoreWakeupSource as LpCoreWakeupSource,
        // WakeupConfig as LpWakeupConfig,
    },
    main,
};
use hil_test::{hp_utils::configure_rtc_pin, ulp_utils::reprogram_ulp_core_with_run_hook};
use lp_app::{
    ULP_BOOT_COUNTER,
    ULP_COMMAND,
    ULP_DEBUG_GPIO_ISR_COUNT,
    ULP_DEBUG_GPIO_ISR_STATUS,
    ULP_DEBUG_LAST_ISR_DATA,
    shared::UlpCmd,
};

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

    // CONFIGURE PIN INTERRUPTS AND GPIO WAKEUP SOURCE
    {
        for pin in 0..=14 {
            // enable all interrupts, no wakeup
            configure_rtc_pin(pin, 3, false, false);
        }
    }

    let mut ulp_core = LpCore::new(peripherals.ULP_RISCV_CORE);
    defmt::warn!("Re-programming ULP!");

    // Run with sleeping, to find out: Do GPIO interrupts fire while asleep?
    // let lp_wake_src = LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(530));
    // let lp_wake_src = LpCoreWakeupSource::Gpio;
    let lp_wake_src = LpCoreWakeupSource::HpCpu;

    reprogram_ulp_core_with_run_hook(&mut ulp_core, lp_wake_src, || {
        ULP_COMMAND.store(UlpCmd::COUNTER_LOOP);
        ULP_DEBUG_GPIO_ISR_STATUS.store(0);
        ULP_DEBUG_LAST_ISR_DATA.store(0);
    });

    defmt::warn!("ULP re-programmed!");

    let dly = esp_hal::delay::Delay::new();

    let mut gpio_isr_count = 0;

    let mut boot_cnt = 0;

    // Print whenever trap or isr data change
    loop {
        let new_boot_counter = ULP_BOOT_COUNTER.load();
        if boot_cnt != new_boot_counter {
            defmt::info!("BOOT COUNT: {}", new_boot_counter);
            boot_cnt = new_boot_counter;
        }

        let new_gpio_isr_count = ULP_DEBUG_GPIO_ISR_COUNT.load();
        let trap_data = ULP_DEBUG_GPIO_ISR_STATUS.load();
        let isr_data = ULP_DEBUG_LAST_ISR_DATA.load();
        if gpio_isr_count != new_gpio_isr_count {
            defmt::info!("({}) TRAP_DATA: 0x{:08x}", new_gpio_isr_count, trap_data);
            defmt::info!("({}) ISR_DATA:  0x{:08x}", new_gpio_isr_count, isr_data);
        }

        gpio_isr_count = new_gpio_isr_count;

        dly.delay_millis(1);
    }
}
