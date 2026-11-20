#![no_std]
#![no_main]

// This library is used by both LP and HP cores.
// It contains the shared global variables!

// Shared variable types, in external crate.
pub use shared::{self, *};

cfg_if::cfg_if! {
if #[cfg(feature = "is-lp-core")] {
    use esp_hal_procmacros::shared_variable;
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
} else {
    // TODO: Use a proc-macro to auto-generate these extern definitions.
    unsafe extern "Rust" {
        pub safe static ULP_HALT_COUNTER: shared::UlpCounter;
        pub safe static ULP_LOOP_COUNTER: shared::UlpCounter;
        pub safe static ULP_BOOT_COUNTER: shared::UlpCounter;
        pub safe static ULP_COMMAND: shared::UlpCmdWrapper;
        pub safe static ULP_REPLY: shared::UlpReplyWrapper;
        pub safe static ULP_LOCK: shared::UlpLockWrapper;
        pub safe static ULP_DEBUG_GPIO_ISR_COUNT: shared::UlpCounter;
        pub safe static ULP_DEBUG_GPIO_ISR_STATUS: shared::UlpCounter;
        pub safe static ULP_DEBUG_LAST_ISR_DATA: shared::UlpCounter;
        pub safe static ULP_TEST_DATA_IN: shared::UlpCounter;
        pub safe static ULP_TEST_DATA_OUT: shared::UlpCounter;
    }
}
}
