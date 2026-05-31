//! Increments a 32 bit counter value at a known point in memory, once a second.
#![no_std]
#![no_main]
#![allow(unused)]
#![allow(static_mut_refs)]

use esp_lp_hal::{
    delay::Delay,
    prelude::*,
    ulp_riscv_halt,
    ulp_riscv_timer_stop,
    ulp_timer_period,
};
use panic_halt as _;
use shared::{
    SharedType,
    TEST_MUTEX_ITERATIONS,
    TEST_XOR_MASK,
    ULP_TEST_DATA_IN,
    ULP_TEST_DATA_OUT,
    UlpCommand,
    UlpLock,
    UlpLoopCounter,
    UlpReply,
};

// This return type is used to indicate if the command should exit the loop or not
#[derive(Clone, Copy, Eq, PartialEq)]
enum CmdResult {
    Continue,
    Break,
}

#[inline(always)]
fn cycles() -> u64 {
    let mut cycles: u32;
    unsafe {
        core::arch::asm!(
            "rdcycle {cycles}",
            cycles = out(reg) cycles,
        )
    }

    cycles as u64
}

fn delay_for_a_tenth_second() {
    const DELAYYY: u64 = 17_500_000 / 10;
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= DELAYYY {}
}

fn delay_for_a_second() {
    const DELAYYY: u64 = 17_500_000 / 1;
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= DELAYYY {}
}

#[entry]
fn main() {
    // The first thing to do is mark us as running
    if UlpReply::load() == UlpReply::UNKNOWN {
        UlpReply::RUNNING.store();
    }

    // let d = Delay {};

    loop {
        // Re-reading the command allows it to be modified by ourself.
        // E.g. MUTEX_TEST will change the command to NOOP when completed,
        // to prevent re-running the test.
        let cmd: UlpCommand = UlpCommand::load();

        match cmd {
            UlpCommand::NOOP => {
                // Do nothing.
            }
            UlpCommand::ONESHOT => {
                // Increment counter ONCE, then stay in a loop
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                UlpCommand::NOOP.store();
            }
            UlpCommand::LOOP_COUNTER_TEST => unsafe {
                // Keep incrementing the counter in a loop
                UlpLoopCounter::increment();
                UlpReply::OK.store();
            },
            UlpCommand::TIMER_COUNTER_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                // Exit the loop, the ULP Timer will re-start us.
                break;
            },
            UlpCommand::XOR_TEST => unsafe {
                UlpLoopCounter::increment();
                let indata = unsafe { ULP_TEST_DATA_IN.clone() };
                unsafe { ULP_TEST_DATA_OUT = indata ^ TEST_XOR_MASK };
                UlpReply::OK.store();
                // Run once.
                UlpCommand::NOOP.store();
            },
            UlpCommand::STOP_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                // Wait so that successful boot can be confirmed
                delay_for_a_second();
                // As a way to check the following functions successfuly halted,
                // we will reset the reply to UNKNOWN state.
                // If the chip re-starts somehow, this will change, detecting a failure to halt.
                UlpReply::UNKNOWN.store();
                // SHOULD never return from here
                ulp_riscv_timer_stop();
                ulp_riscv_halt();
            },
            UlpCommand::MUTEX_TEST => unsafe {
                for _ in 0..TEST_MUTEX_ITERATIONS {
                    UlpLock::acquire();
                    UlpLoopCounter::increment();
                    UlpLock::release();
                }
                UlpReply::OK.store();
                // Run once
                UlpCommand::NOOP.store();
            },
            UlpCommand::TIMER_PERIOD_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpLock::acquire();
                let new_cycles = unsafe { ULP_TEST_DATA_IN.clone() };
                UlpLock::release();
                ulp_timer_period(new_cycles);
                UlpReply::OK.store();
                break;
            },
            _ => unsafe {
                // Unknown command, not okay!
                UlpReply::NOK.store();
            },
        };
    }
}
