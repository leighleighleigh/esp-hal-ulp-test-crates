//! Increments a 32 bit counter value at a known point in memory, once a second.
#![no_std]
#![no_main]
#![allow(unused)]

use esp_lp_hal::{delay::Delay, prelude::*};
use panic_halt as _;
use shared::{UlpCommand, UlpCommandType, UlpLoopCounter, UlpReply, UlpReplyType};

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

fn delay_for_a_tenth_second()
{
    const DELAYYY : u64 = 17_500_000 / 10;
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= DELAYYY {}
}

#[entry]
fn main() {
    // Handle command
    let cmd: UlpCommandType = UlpCommand::read();
    // Loop until command says to stop
    let mut has_incremented = false;

    loop {
        match cmd {
            UlpCommandType::NOOP => {
                // Increment counter ONCE, then stay in a loop
                if !has_incremented {
                    UlpLoopCounter::increment();
                    has_incremented = true;
                }
                UlpReply::write(UlpReplyType::OK);
            },
            UlpCommandType::LOOP_COUNTER_TEST => {
                // Blocking counter in the loop
                UlpLoopCounter::increment();
                UlpReply::write(UlpReplyType::OK);
                // dly.delay_millis(1000);
                // delay_for_a_tenth_second();
            },
            UlpCommandType::TIMER_COUNTER_TEST => {
                UlpLoopCounter::increment();
                UlpReply::write(UlpReplyType::OK);
                break;
            },
            _ => {
                // Loop forever but dont increment
                UlpReply::write(UlpReplyType::UNIMPLEMENTED);
            }
        };
    }
}
