#![no_std]

// This library is used by both LP and HP cores.
// It contains the shared global variables!
use esp_hal_procmacros::shared_variable;
// Shared variable types, in external crate.
pub use shared;

#[shared_variable]
pub static ULP_HALT_COUNTER: shared::UlpCounter = shared::UlpCounter::new();
#[shared_variable]
pub static ULP_LOOP_COUNTER: shared::UlpCounter = shared::UlpCounter::new();
#[shared_variable]
pub static ULP_BOOT_COUNTER: shared::UlpCounter = shared::UlpCounter::new();
#[shared_variable]
pub static ULP_COMMAND: shared::UlpCmdWrapper = shared::UlpCmdWrapper::new();
#[shared_variable]
pub static ULP_REPLY: shared::UlpReplyWrapper = shared::UlpReplyWrapper::new();
#[shared_variable]
pub static ULP_LOCK: shared::UlpLockWrapper = shared::UlpLockWrapper::new();
#[shared_variable]
pub static ULP_DEBUG_GPIO_ISR_COUNT: shared::UlpCounter = shared::UlpCounter::new();
#[shared_variable]
pub static ULP_DEBUG_GPIO_ISR_STATUS: shared::UlpCounter = shared::UlpCounter::new();
#[shared_variable]
pub static ULP_DEBUG_LAST_ISR_DATA: shared::UlpCounter = shared::UlpCounter::new();
#[shared_variable]
pub static ULP_TEST_DATA_IN: shared::UlpCounter = shared::UlpCounter::new();
#[shared_variable]
pub static ULP_TEST_DATA_OUT: shared::UlpCounter = shared::UlpCounter::new();

// This file is written by build.rs
#[cfg(not(feature = "is-lp-core"))]
include!(concat!(env!("OUT_DIR"), "/load_macro.rs"));
