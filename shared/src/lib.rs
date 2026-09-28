#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod traits;
pub use traits::{SharedType, SharedTypeConversion};
mod types;
pub use types::*;
mod locks;
pub use locks::UlpLock;

pub const TEST_XOR_MASK: u32 = 0xcafe;
pub const TEST_MUTEX_ITERATIONS: u32 = 1000;

cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_LOCK: UlpLock = UlpLock::new();

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_COMMAND: UlpCommand = UlpCommand::UNSET;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_REPLY: UlpReply = UlpReply::UNSET;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_BOOT_COUNTER: u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_HALT_COUNTER: u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_LOOP_COUNTER: u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_TEST_DATA_IN : u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_TEST_DATA_OUT : u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_DEBUG_GPIO_ISR_COUNT: u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_DEBUG_GPIO_ISR_STATUS : u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_DEBUG_LAST_ISR_DATA : u32 = 0;

    } else {
        unsafe extern "Rust" {
            pub static mut ULP_LOCK: UlpLock;
            pub static mut ULP_COMMAND: UlpCommand;
            pub static mut ULP_REPLY: UlpReply;
            pub static mut ULP_BOOT_COUNTER: u32;
            pub static mut ULP_HALT_COUNTER: u32;
            pub static mut ULP_LOOP_COUNTER: u32;
            pub static mut ULP_TEST_DATA_IN : u32;
            pub static mut ULP_TEST_DATA_OUT : u32;
            pub static mut ULP_DEBUG_GPIO_ISR_COUNT: u32;
            pub static mut ULP_DEBUG_GPIO_ISR_STATUS: u32;
            pub static mut ULP_DEBUG_LAST_ISR_DATA : u32;
        }
    }
}
