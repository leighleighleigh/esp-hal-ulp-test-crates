//! Increments a 32 bit counter value at a known point in memory, once a second.
#![no_std]
#![no_main]
#![allow(unused)]
#![allow(static_mut_refs)]

use esp_lp_hal::{delay::Delay, prelude::*};
use panic_halt as _;
use shared::{
    SharedType,
    UlpLoopCounter,
    UlpCommand,
    UlpReply,
    TEST_XOR_MASK,
    ULP_TEST_DATA_IN,
    ULP_TEST_DATA_OUT,
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

#[entry]
fn main() {
    let cmd: UlpCommand = UlpCommand::load();
    let mut has_incremented = false;

    loop {
        match cmd {
            UlpCommand::NOOP => {
                // Increment counter ONCE, then stay in a loop
                if !has_incremented {
                    UlpLoopCounter::increment();
                    has_incremented = true;
                }
                UlpReply::OK.store();
            }
            UlpCommand::LOOP_COUNTER_TEST => unsafe {
                // Blocking counter in the loop
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                // dly.delay_millis(1000);
                // delay_for_a_tenth_second();
            },
            UlpCommand::TIMER_COUNTER_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                break;
            },
            UlpCommand::XOR_TEST => unsafe {
                let indata = unsafe { ULP_TEST_DATA_IN.clone() };
                unsafe { ULP_TEST_DATA_OUT = indata ^ TEST_XOR_MASK };
                UlpReply::OK.store();
            },
            UlpCommand::STOP_TEST => unsafe {
                UlpReply::UNIMPLEMENTED.store();
            },
            _ => unsafe {
                // Unknown command, not okay!
                UlpReply::NOK.store();
            },
        };
    }
}
