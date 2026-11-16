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

pub mod locks;
pub mod traits;
pub mod types;

pub use locks::{UlpLockInner, UlpLockWrapper};
pub use traits::SharedCell;
pub use types::{UlpCmd, UlpCmdWrapper, UlpCounter, UlpReply, UlpReplyWrapper};

pub const TEST_XOR_MASK: u32 = 0xcafe;
pub const TEST_MUTEX_ITERATIONS: u32 = 1000;
