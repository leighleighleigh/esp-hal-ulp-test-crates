//! Increments a 32 bit counter value at a known point in memory, once a second.

#![no_std]
#![no_main]

use esp_lp_hal::{delay::Delay, prelude::*};
use panic_halt as _;
use shared::{UlpCommand, UlpCommandType, UlpLoopCounter, UlpReply, UlpReplyType};

// This return type is used to indicate if the command should exit the loop or not
enum CmdResult {
    Continue,
    Break,
}

fn handle_command(cmd: UlpCommandType) -> CmdResult {
    let dly = Delay {};

    match cmd {
        UlpCommandType::RISCV_UNKNOWN_COMMAND => {
            UlpReply::write(UlpReplyType::RISCV_COMMAND_UNKNOWN);
            CmdResult::Continue
        }
        UlpCommandType::RISCV_COUNTER_TEST => {
            // Blocking counter in the loop
            UlpReply::write(UlpReplyType::RISCV_COMMAND_OK);
            dly.delay_millis(100);
            UlpLoopCounter::increment();
            CmdResult::Continue
        }
        UlpCommandType::RISCV_ULP_TIMER_COUNTER_TEST => {
            UlpReply::write(UlpReplyType::RISCV_COMMAND_OK);
            CmdResult::Break
        }
        UlpCommandType::RISCV_NO_COMMAND => {
            UlpReply::write(UlpReplyType::RISCV_COMMAND_OK);
            CmdResult::Break
        }
    }
}

#[entry]
fn main() {
    // Increment counter on boot
    UlpLoopCounter::increment();

    // Handle command
    let cmd: UlpCommandType = UlpCommand::read();

    // Loop until command says to stop
    while let CmdResult::Continue = handle_command(cmd) {}
}
