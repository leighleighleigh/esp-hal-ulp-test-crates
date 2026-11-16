#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![feature(sync_unsafe_cell)]
#![cfg_attr(target_arch = "xtensa", feature(asm_experimental_arch))]

#[allow(unused)]
#[cfg(feature = "defmt")]
use defmt;

pub mod traits;
mod types;
pub use types::*;
mod locks;
pub use locks::*;

pub const TEST_XOR_MASK: u32 = 0xcafe;
pub const TEST_MUTEX_ITERATIONS: u32 = 1000;

// NOTE: Below is temporary, provides HP with all extern-ed variables. These will be PROVIDE-ed by
// the lp_app.x file.       This extern block will be auto-generated using a different macro in
// future, or maybe hidden behind a shared type or something, idk yet.
#[cfg(not(feature = "is-lp-core"))]
unsafe extern "Rust" {
    // mark these as 'safe' so Rust doesn't freak out,
    // allowing it to be treated like a regular global static.
    pub safe static ULP_LOCK: UlpLockWrapper;
    pub safe static ULP_COMMAND: UlpCmdWrapper;
    pub safe static ULP_REPLY: UlpReplyWrapper;
    pub safe static ULP_BOOT_COUNTER: UlpCounter;
    pub safe static ULP_HALT_COUNTER: UlpCounter;
    pub safe static ULP_LOOP_COUNTER: UlpCounter;
    pub safe static mut ULP_TEST_DATA_IN: u32;
    pub safe static mut ULP_TEST_DATA_OUT: u32;
    pub safe static mut ULP_DEBUG_GPIO_ISR_COUNT: u32;
    pub safe static mut ULP_DEBUG_GPIO_ISR_STATUS: u32;
    pub safe static mut ULP_DEBUG_LAST_ISR_DATA: u32;
}
