#![allow(unused)]
#![allow(static_mut_refs)]
use esp_lp_hal::prelude::shared;

use super::{UlpBootCounter, UlpCommand, UlpHaltCounter, UlpLock, UlpLoopCounter, UlpReply};

#[shared]
pub static mut ULP_BOOT_COUNTER: u32 = 0;
#[shared]
pub static mut ULP_COMMAND: UlpCommand = UlpCommand::NOOP;
#[shared]
pub static mut ULP_DEBUG_GPIO_ISR_COUNT: u32 = 0;
#[shared]
pub static mut ULP_DEBUG_GPIO_ISR_STATUS: u32 = 0;
#[shared]
pub static mut ULP_DEBUG_LAST_ISR_DATA: u32 = 0;
#[shared]
pub static mut ULP_HALT_COUNTER: u32 = 0;
#[shared]
pub static mut ULP_LOCK: UlpLock = UlpLock::new();
#[shared]
pub static mut ULP_LOOP_COUNTER: u32 = 0;
#[shared]
pub static mut ULP_REPLY: UlpReply = UlpReply::UNSET;
#[shared]
pub static mut ULP_TEST_DATA_IN: u32 = 0;
#[shared]
pub static mut ULP_TEST_DATA_OUT: u32 = 0;
