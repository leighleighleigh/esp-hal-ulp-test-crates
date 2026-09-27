#![no_std]
#![no_main]

use esp32s3 as pac;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    lp_core::{
        UlpCore as LpCore,
        UlpCoreTimerCycles as LpCoreTimerCycles,
        UlpCoreWakeupSource as LpCoreWakeupSource,
        // WakeupConfig as LpWakeupConfig,
    },
    main,
};
use esp_println as _;
use hil_test::{hp_utils::configure_rtc_pin, ulp_utils::reprogram_ulp_core_with_run_hook};
use shared::{
    SharedType,
    UlpBootCounter,
    UlpCommand,
    UlpHaltCounter,
    ULP_DEBUG_GPIO_ISR_COUNT,
    ULP_DEBUG_GPIO_ISR_STATUS,
    ULP_DEBUG_LAST_ISR_DATA,
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
            configure_rtc_pin(pin, 3, false);
        }
    }

    let mut ulp_core = LpCore::new(peripherals.ULP_RISCV_CORE);
    defmt::warn!("Re-programming ULP!");

    // Run with sleeping, to find out: Do GPIO interrupts fire while asleep?
    // let lp_wake_src = LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(530));
    // let lp_wake_src = LpCoreWakeupSource::Gpio;
    let lp_wake_src = LpCoreWakeupSource::HpCpu;

    reprogram_ulp_core_with_run_hook(&mut ulp_core, lp_wake_src, || {
        UlpCommand::COUNTER_LOOP.store();
        unsafe { ULP_DEBUG_GPIO_ISR_STATUS = 0x0 };
        unsafe { ULP_DEBUG_LAST_ISR_DATA = 0x0 };
    });

    defmt::warn!("ULP re-programmed!");

    let dly = esp_hal::delay::Delay::new();

    let mut gpio_isr_count = 0;

    let mut boot_cnt = 0;

    // Print whenever trap or isr data change
    loop {
        let new_boot_counter = UlpBootCounter::load();
        if boot_cnt != new_boot_counter {
            defmt::info!("BOOT COUNT: {}", new_boot_counter);
            boot_cnt = new_boot_counter;
        }

        let new_gpio_isr_count = unsafe { ULP_DEBUG_GPIO_ISR_COUNT };
        let trap_data = unsafe { ULP_DEBUG_GPIO_ISR_STATUS };
        let isr_data = unsafe { ULP_DEBUG_LAST_ISR_DATA };
        if gpio_isr_count != new_gpio_isr_count {
            defmt::info!("({}) TRAP_DATA: 0x{:08x}", new_gpio_isr_count, trap_data);
            defmt::info!("({}) ISR_DATA:  0x{:08x}", new_gpio_isr_count, isr_data);
            // let new_boot_cnt = UlpBootCounter::load();
            // let new_halt_cnt = UlpHaltCounter::load();
            // defmt::info!(
            //     "({}) Boot count: 0x{:08x}",
            //     new_gpio_isr_count,
            //     new_boot_cnt
            // );
            // defmt::info!(
            //     "({}) Halt count: 0x{:08x}",
            //     new_gpio_isr_count,
            //     new_halt_cnt
            // );
        }

        gpio_isr_count = new_gpio_isr_count;

        dly.delay_millis(1);
    }
}
