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

#[entry]
fn main() {
   // const DELAYYY : u64 = 175000; 
   // let t0 = cycles();
   // while cycles().wrapping_sub(t0) <= DELAYYY {}

   // Handle command
   // let cmd: UlpCommandType = UlpCommand::read();

   // UlpReply::write(UlpReplyType::REPLY_OK);

   UlpLoopCounter::increment();

   
   // Loop until command says to stop
   // let mut has_incremented = false;

   // loop {
   //     match cmd {
   //         UlpCommandType::NOOP => {
   //             // Increment counter ONCE, then stay in a loop
   //             if !has_incremented {
   //                 UlpLoopCounter::increment();
   //                 has_incremented = true;
   //             }
   //             // UlpReply::write(UlpReplyType::REPLY_OK);
   //         },
   //         UlpCommandType::LOOP_COUNTER_TEST => {
   //             // Blocking counter in the loop
   //             UlpLoopCounter::increment();
   //             // UlpReply::write(UlpReplyType::REPLY_OK);
   //             // dly.delay_millis(100);
   //             // dly.delay_nanos(100);
   //             const CLOCK : u64 = 1750000*2; 
   //             let t0 = cycles();
   //             while cycles().wrapping_sub(t0) <= CLOCK {}
   //         },
   //         UlpCommandType::TIMER_COUNTER_TEST => {
   //             UlpLoopCounter::increment();
   //             // UlpReply::write(UlpReplyType::REPLY_OK);
   //             break;
   //         },
   //         _ => {
   //             // Loop forever but dont increment
   //             // UlpReply::write(UlpReplyType::REPLY_UNKNOWN);
   //         }
   //     };
   // }
}

