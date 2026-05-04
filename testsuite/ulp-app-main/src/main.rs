//! Increments a 32 bit counter value at a known point in memory, once a second.

#![no_std]
#![no_main]

use esp_lp_hal::prelude::*;
use esp_lp_hal::delay::Delay;
use panic_halt as _;

use shared::{UlpLoopCounter, UlpCommand, UlpReply, RW};

fn increment_counter() {
    let mut count : u32 = UlpLoopCounter::load().into_raw();
    count = count.wrapping_add(1u32);
    UlpLoopCounter::from_raw(count).save();
}

#[entry]
fn main() {
    // Increment counter on boot
    increment_counter();

    // Handle command
    let dly = Delay {};
    let cmd : UlpCommand = UlpCommand::load();

    loop {
      match cmd {
        UlpCommand::RISCV_COUNTER_TEST => {
          // command is ok
          UlpReply::RISCV_COMMAND_OK.save();
          // run in the loop, incrementing and delaying
          dly.delay_millis(100);
          increment_counter();
        },
        UlpCommand::RISCV_ULP_TIMER_COUNTER_TEST => {
          UlpReply::RISCV_COMMAND_OK.save();
          // break! only increment on boot
          break
        },
        UlpCommand::RISCV_NO_COMMAND => {
          UlpReply::RISCV_COMMAND_OK.save();
          break;
        }, // loop forever, no increment
        UlpCommand::RISCV_UNKNOWN_COMMAND(cmdval) => {
          UlpReply::RISCV_COMMAND_UNKNOWN(cmdval).save();
        }
      }
    }
}
