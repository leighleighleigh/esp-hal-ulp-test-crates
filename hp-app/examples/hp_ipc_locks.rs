#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    lp_core::{UlpCore as LpCore, UlpCoreWakeupSource as LpCoreWakeupSource},
    main,
    time::Instant,
};
use hil_test::{info, setup_logger, ulp_utils::reprogram_ulp_core_with_run_hook, warn};
use lp_app::{
    ULP_COMMAND,
    ULP_LOCK,
    ULP_LOOP_COUNTER,
    ULP_REPLY,
    ULP_TEST_DATA_IN,
    shared::{UlpCmd, UlpReply},
};
use num_traits::Float;
const MUTEX_ITERATIONS: u32 = 10000;
const HP_CORE_INCREMENTS: bool = true;

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    setup_logger();
    info!("Boot!");

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
        ULP_COMMAND.store(UlpCmd::MUTEX_TEST);
        ULP_TEST_DATA_IN.store(MUTEX_ITERATIONS);
    });

    let mut superloop = 0;

    // Keep a running average, and also keep track of the variables and sample uncertainty.
    let mut test_samples: [f64; _] = [0.0; 100];

    loop {
        info!("Starting superloop {}", superloop);
        let t0 = Instant::now();

        for _i in 0..MUTEX_ITERATIONS {
            ULP_LOCK.acquire();
            if HP_CORE_INCREMENTS {
                ULP_LOOP_COUNTER.increment();
            }
            ULP_LOCK.release();
        }

        // Also acquire/release the lock while we wait for ULP to finish
        while ULP_REPLY.load() != UlpReply::OK {
            ULP_LOCK.acquire();
            ULP_LOCK.release();
        }

        let test_duration = t0.elapsed();
        let test_rate = (MUTEX_ITERATIONS as u64 * 1000000) / test_duration.as_micros();
        test_samples[superloop % test_samples.len()] = test_rate as f64;

        if HP_CORE_INCREMENTS {
            // Assert no race conditions and we incremented 2x the number of loops
            hil_test::assert_eq!(2 * MUTEX_ITERATIONS, ULP_LOOP_COUNTER.load());
        } else {
            // Assert the ULP core incremented 1x the number of loops
            hil_test::assert_eq!(MUTEX_ITERATIONS, ULP_LOOP_COUNTER.load());
        }

        // Calculate mean and sample variance
        let n = (superloop + 1).min(100);
        if n >= 2 {
            let mut test_sample_mu = 0.0;
            for i in 0..n {
                test_sample_mu += test_samples[i];
            }
            test_sample_mu /= n as f64;
            // warn!("Loop {} mean {}", superloop, test_sample_mu as u64);

            let mut test_sample_var: f64 = 0.0;
            for i in 0..n {
                let x = test_samples[i] - test_sample_mu;
                let xx = x * x;
                test_sample_var += xx;
            }
            test_sample_var /= (n - 1) as f64;
            let test_sample_std = test_sample_var.powf(0.5);

            warn!(
                "Loop {} did {} iters in {} ({} iter/sec, mean {}, std {})",
                superloop,
                MUTEX_ITERATIONS,
                test_duration,
                test_rate,
                test_sample_mu as u64,
                test_sample_std as u64,
            );
        } else {
            warn!(
                "Loop {} did {} iters in {} ({} iter/sec)",
                superloop, MUTEX_ITERATIONS, test_duration, test_rate
            );
        }

        superloop += 1;

        dly.delay_millis(10); // wait for a second before running again

        // Reset the counter
        ULP_LOOP_COUNTER.reset();
        // Tell the core to do the test again
        ULP_COMMAND.store(UlpCmd::MUTEX_TEST);
    }
}
