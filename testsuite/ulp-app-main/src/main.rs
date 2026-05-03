//! Increments a 32 bit counter value at a known point in memory, once a second.

#![no_std]
#![no_main]

use esp_lp_hal::prelude::*;
use esp_lp_hal::delay::Delay;
use panic_halt as _;

use shared::{COMMAND_ADDRESS, COUNTER_ADDRESS, UlpCommand, UlpReply, reg_read, reg_write, RW};

fn increment_counter() {
    let mut count : u32 = reg_read(COUNTER_ADDRESS);
    count = count.wrapping_add(1u32);
    reg_write(COUNTER_ADDRESS, count);
}

#[entry]
fn main() {
    // Increment counter on boot
    increment_counter();

    // Handle command
    let dly = Delay {};
    let cmd : UlpCommand = UlpCommand::read();

    loop {
      match cmd {
        UlpCommand::RISCV_COUNTER_TEST => {
          // command is ok
          UlpReply::RISCV_COMMAND_OK.write();
          // run in the loop, incrementing and delaying
          dly.delay_millis(100);
          increment_counter();
        },
        UlpCommand::RISCV_ULP_TIMER_COUNTER_TEST => {
          UlpReply::RISCV_COMMAND_OK.write();
          // break! only increment on boot
          break
        },
        UlpCommand::RISCV_NO_COMMAND => {
          UlpReply::RISCV_COMMAND_OK.write();
          // reg_write(COMMAND_ADDRESS, cmd.into_raw());
          break;
        }, // loop forever, no increment
        UlpCommand::RISCV_UNKNOWN_COMMAND(cmdval) => {
          UlpReply::RISCV_COMMAND_UNKNOWN(cmdval).write();
        }
      }
    }
}
