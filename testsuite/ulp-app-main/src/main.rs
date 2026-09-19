//! Increments a 32 bit counter value at a known point in memory, once a second.
#![no_std]
#![no_main]
#![allow(unused)]
#![allow(static_mut_refs)]

use core::iter;

use esp_lp_hal::{delay::Delay, prelude::*, ulp_riscv_timer_stop, ulp_timer_period, wake_hp_core,
    interrupt::{Exception, ExternalInterrupt, exception, external_interrupt, TrapFrame},
};

use panic_halt as _;
use shared::{
    SharedType,
    UlpBootCounter,
    UlpCommand,
    UlpHaltCounter,
    UlpLock,
    UlpLoopCounter,
    UlpReply,
    TEST_MUTEX_ITERATIONS,
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

#[inline]
fn delay_using_loop(iterations: u32) {
    for i in 0..iterations {
        unsafe {
            core::arch::asm!("nop");
        }
    }
}

fn delay_for_a_hundred_microseconds() {
    const DELAYYY: u64 = 1750;
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= DELAYYY {}
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
    UlpBootCounter::increment();

    loop {
        // Re-reading the command allows it to be modified by ourself.
        // E.g. MUTEX_TEST will change the command to NOOP when completed,
        // to prevent re-running the test.
        let cmd: UlpCommand = UlpCommand::load();

        match cmd {
            UlpCommand::NOOP => {
                // Do nothing
                UlpReply::OK.store();
            }
            UlpCommand::COUNTER_ONESHOT => {
                // Increment counter ONCE, then stay loop-ing on NOOP.
                UlpLoopCounter::increment();
                UlpCommand::NOOP.store();
                UlpReply::OK.store();
            }
            UlpCommand::COUNTER_LOOP => unsafe {
                // Keep incrementing the counter in a loop
                UlpLoopCounter::increment();
                UlpReply::OK.store();
            },
            UlpCommand::COUNTER_ULP_TIMER => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                // Exit the loop, the ULP Timer will re-start us.
                break;
            },
            UlpCommand::XOR_TEST => unsafe {
                UlpLoopCounter::increment();
                let data_in = unsafe { ULP_TEST_DATA_IN.clone() };
                unsafe { ULP_TEST_DATA_OUT = data_in ^ TEST_XOR_MASK };
                // Run once.
                UlpCommand::NOOP.store();
                UlpReply::OK.store();
            },
            UlpCommand::STOP_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                ulp_riscv_timer_stop();
                break;
            },
            UlpCommand::MUTEX_TEST => unsafe {
                for _ in 0..TEST_MUTEX_ITERATIONS {
                    UlpLock::acquire();
                    UlpLoopCounter::increment();
                    UlpLock::release();
                }
                UlpCommand::NOOP.store();
                UlpReply::OK.store();
            },
            UlpCommand::TIMER_PERIOD_TEST => unsafe {
                UlpLoopCounter::increment();
                let new_cycles = unsafe { ULP_TEST_DATA_IN.clone() };
                ulp_timer_period(new_cycles);
                UlpReply::OK.store();
                break;
            },
            UlpCommand::LIGHT_SLEEP_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();

                // NEW approach - wait for HP core to release the lock.
                // HP core has 0.1 seconds to acquire before we do.
                delay_for_a_tenth_second();
                // BLOCK HERE UNTIL HP CORE RELEASES
                UlpLock::acquire();
                // Wake up the core
                wake_hp_core();
                // Disable the timer, so the LP-core will remain halted on exit.
                ulp_riscv_timer_stop();
                break;
            },
            UlpCommand::EXCEPTION_TEST => unsafe {
                // Should trigger an illegal instruction
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                unsafe{
                    core::arch::asm!(
                        "csrrs a1, mcause, zero"
                    );
                }
                break;
            }
            _ => unsafe {
                // Unknown command, not okay!
                UlpReply::NOK.store();
            },
        };
    }

    UlpHaltCounter::increment();
}

// Used for EXCEPTION_TEST
#[exception(Exception::IllegalInstruction)]
unsafe fn illegal_instruction(_trap: &TrapFrame) -> ! {
    unsafe { ULP_TEST_DATA_OUT = 0xdeadbeef };
    loop {}
}