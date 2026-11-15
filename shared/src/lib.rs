#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(target_arch = "xtensa", feature(asm_experimental_arch))]

#[cfg(feature = "defmt")]
use defmt;

pub mod traits;
pub use traits::{SharedType, SharedTypeConversion, SharedVar};
mod types;
pub use types::*;
mod locks;
pub use locks::*;

pub const TEST_XOR_MASK: u32 = 0xcafe;
pub const TEST_MUTEX_ITERATIONS: u32 = 1000;

// NOTE: Below is temporary but it does work lol, allows the LP core to link against itself.
#[cfg(feature = "is-lp-core")]
unsafe extern "Rust" {
    #[link_name = "__ULP_$ULP_LOCK$UlpLock"]
    pub static mut ULP_LOCK: UlpLock;
    #[link_name = "__ULP_$ULP_COMMAND$UlpCommand"]
    pub static mut ULP_COMMAND: UlpCommand;
    #[link_name = "__ULP_$ULP_REPLY$UlpReply"]
    pub static mut ULP_REPLY: UlpReply;
    #[link_name = "__ULP_$ULP_BOOT_COUNTER$u32"]
    pub static mut ULP_BOOT_COUNTER: u32;
    #[link_name = "__ULP_$ULP_HALT_COUNTER$u32"]
    pub static mut ULP_HALT_COUNTER: u32;
    #[link_name = "__ULP_$ULP_LOOP_COUNTER$u32"]
    pub static mut ULP_LOOP_COUNTER: u32;
}

// NOTE: Below is temporary, provides HP with all extern-ed variables. These will be PROVIDE-ed by
// the lp_app.x file.       This extern block will be auto-generated using a different macro in
// future, or maybe hidden behind a shared type or something, idk yet.
#[cfg(not(feature = "is-lp-core"))]
unsafe extern "Rust" {
    pub static mut ULP_LOCK: UlpLock;
    pub static mut ULP_COMMAND: UlpCommand;
    pub static mut ULP_REPLY: UlpReply;
    pub static mut ULP_BOOT_COUNTER: u32;
    pub static mut ULP_HALT_COUNTER: u32;
    pub static mut ULP_LOOP_COUNTER: u32;
    pub static mut ULP_TEST_DATA_IN: u32;
    pub static mut ULP_TEST_DATA_OUT: u32;
    pub static mut ULP_DEBUG_GPIO_ISR_COUNT: u32;
    pub static mut ULP_DEBUG_GPIO_ISR_STATUS: u32;
    pub static mut ULP_DEBUG_LAST_ISR_DATA: u32;
}
