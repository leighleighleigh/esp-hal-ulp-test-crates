#![allow(unused)]
#![allow(static_mut_refs)]

use esp_lp_hal::prelude::shared;
use sharedtypes::{
    UlpCmd,
    UlpCmdWrapper,
    UlpCounter,
    UlpLockInner,
    UlpLockWrapper,
    UlpReply,
    UlpReplyWrapper,
};

#[shared]
pub static ULP_HALT_COUNTER: UlpCounter = UlpCounter::new(0);
#[shared]
pub static ULP_LOOP_COUNTER: UlpCounter = UlpCounter::new(0);
#[shared]
pub static ULP_BOOT_COUNTER: UlpCounter = UlpCounter::new(0);
#[shared]
pub static ULP_COMMAND: UlpCmdWrapper = UlpCmdWrapper::new(UlpCmd::NOOP);
#[shared]
pub static ULP_REPLY: UlpReplyWrapper = UlpReplyWrapper::new(UlpReply::UNSET);
#[shared]
pub static ULP_LOCK: UlpLockWrapper = UlpLockWrapper::new(UlpLockInner::new());

#[shared]
pub static mut ULP_DEBUG_GPIO_ISR_COUNT: u32 = 0;
#[shared]
pub static mut ULP_DEBUG_GPIO_ISR_STATUS: u32 = 0;
#[shared]
pub static mut ULP_DEBUG_LAST_ISR_DATA: u32 = 0;
#[shared]
pub static mut ULP_TEST_DATA_IN: u32 = 0;
#[shared]
pub static mut ULP_TEST_DATA_OUT: u32 = 0;
