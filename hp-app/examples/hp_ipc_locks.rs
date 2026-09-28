#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    lp_core::{UlpCore as LpCore, UlpCoreWakeupSource as LpCoreWakeupSource},
    main,
    time::Instant,
};
use esp_println as _;
use hil_test::ulp_utils::reprogram_ulp_core_with_run_hook;
use shared::{ULP_TEST_DATA_IN, UlpCommand, UlpLock, UlpLoopCounter, UlpReply, traits::*};

const MUTEX_ITERATIONS: u32 = 10000;
const HP_CORE_INCREMENTS: bool = false;

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

    let dly = esp_hal::delay::Delay::new();

    // Program the core once
    let mut ulp_core = LpCore::new(peripherals.ULP_RISCV_CORE);
    let lp_wake_src = LpCoreWakeupSource::HpCpu;
    reprogram_ulp_core_with_run_hook(&mut ulp_core, lp_wake_src, || {
        UlpCommand::MUTEX_TEST.store();
        unsafe { ULP_TEST_DATA_IN = MUTEX_ITERATIONS };
    });

    let mut superloop = 0;

    loop {
        let t0 = Instant::now();

        for _i in 0..MUTEX_ITERATIONS {
            UlpLock::acquire();
            if HP_CORE_INCREMENTS {
                UlpLoopCounter::increment();
            }
            UlpLock::release();
        }

        // Also acquire/release the lock while we wait for ULP to finish
        while UlpReply::load() != UlpReply::OK {
            UlpLock::acquire();
            UlpLock::release();
        }

        let test_duration = t0.elapsed();
        let test_rate = (MUTEX_ITERATIONS as u64 * 1000000) / test_duration.as_micros();

        defmt::warn!(
            "Loop {} did {} iters in {} ({} iter/sec)",
            superloop,
            MUTEX_ITERATIONS,
            test_duration,
            test_rate
        );

        if HP_CORE_INCREMENTS {
            // Assert no race conditions and we incremented 2x the number of loops
            hil_test::assert_eq!(2 * MUTEX_ITERATIONS, UlpLoopCounter::load());
        } else {
            // Assert the ULP core incremented 1x the number of loops
            hil_test::assert_eq!(MUTEX_ITERATIONS, UlpLoopCounter::load());
        }

        superloop += 1;

        dly.delay_millis(1000); // wait for a second before running again

        // Reset the counter
        UlpLoopCounter::reset();
        // Tell the core to do the test again
        UlpCommand::MUTEX_TEST.store();
    }
}
