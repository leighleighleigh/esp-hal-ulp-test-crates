#![no_std]

// This library is used by both LP and HP cores.
// It contains the shared global variables!
use esp_hal_procmacros::shared_variable;

#[shared_variable]
pub static mut RAINBOW_COUNTER: u32 = 0;
#[shared_variable]
pub static mut RAINBOW_RUN: u32 = 0;
#[shared_variable]
pub static mut OTHER_VAR: u32 = 0;

// This file is written by build.rs
#[cfg(not(feature = "is-lp-core"))]
include!(concat!(env!("OUT_DIR"), "/load_macro.rs"));
