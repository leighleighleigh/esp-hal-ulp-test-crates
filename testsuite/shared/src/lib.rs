#![no_std]
#![no_main]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]

mod locks;
pub use locks::{ulp_riscv_lock_acquire, ulp_riscv_lock_release};

pub const TEST_XOR_MASK: u32 = 0xcafe;

cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_COMMAND: UlpCommand = UlpCommand::UNKNOWN;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_REPLY: UlpReply = UlpReply::UNKNOWN;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_LOOP_COUNTER: UlpLoopCounter = UlpLoopCounter::new(0);

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_TEST_DATA_IN : u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_TEST_DATA_OUT : u32 = 0;
    } else {
        unsafe extern "Rust" {
            pub static mut ULP_COMMAND: UlpCommand;
            pub static mut ULP_REPLY: UlpReply;
            pub static mut ULP_LOOP_COUNTER: UlpLoopCounter;
            pub static mut ULP_TEST_DATA_IN : u32;
            pub static mut ULP_TEST_DATA_OUT : u32;
        }
    }
}

// Trait for these unique shared variable types
pub trait SharedType {
    const BACKING_VAR: *const Self;

    // Load a value from memory
    fn load() -> Self
    where
        Self: Sized,
    {
        unsafe {
            let p = <Self as SharedType>::BACKING_VAR;
            p.read_unaligned()
        }
    }

    // Store a value to memory
    fn store(self: Self)
    where
        Self: Sized,
    {
        unsafe {
            let ptr = <Self as SharedType>::BACKING_VAR;
            let mut_ptr = ptr.cast_mut();
            mut_ptr.write_volatile(self);
        }
    }
}

// trait SharedType {
//     type Type;
//     const BACKING_VAR: *const Self::Type;

//     fn load() -> Self::Type {
//         unsafe {
//             let p = <Self as SharedType>::BACKING_VAR;
//             p.read_unaligned()
//         }
//     }

//     fn store(value: Self::Type) {
//         unsafe {
//             let ptr = <Self as SharedType>::BACKING_VAR;
//             let mut_ptr = ptr.cast_mut();
//             mut_ptr.write_volatile(value);
//         }
//     }

//     fn apply(self)
//     where
//         Self: Sized,
//     {
//         Self::store(self);
//     }
// }

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpCommand {
    UNKNOWN            = 0,
    NOOP               = 1,
    LOOP_COUNTER_TEST  = 2,
    TIMER_COUNTER_TEST = 3,
    XOR_TEST           = 4,
    // RISCV_DEEP_SLEEP_WAKEUP_SHORT_DELAY_TEST,
    // RISCV_DEEP_SLEEP_WAKEUP_LONG_DELAY_TEST,
    // RISCV_LIGHT_SLEEP_WAKEUP_TEST,
    STOP_TEST          = 5,
    MUTEX_TEST         = 6,
}

impl SharedType for UlpCommand {
    const BACKING_VAR: *const Self = unsafe { &ULP_COMMAND };
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpReply {
    UNKNOWN       = 0,
    OK            = 1,
    NOK           = 2,
    UNIMPLEMENTED = 3,
}

impl SharedType for UlpReply {
    const BACKING_VAR: *const Self = unsafe { &ULP_REPLY };
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
#[non_exhaustive]
pub struct UlpLoopCounter(u32);

impl From<u32> for UlpLoopCounter {
    fn from(value: u32) -> Self {
        Self { 0: value }
    }
}

impl Into<u32> for UlpLoopCounter {
    fn into(self) -> u32 {
        self.0
    }
}

impl SharedType for UlpLoopCounter {
    const BACKING_VAR: *const Self = unsafe { &ULP_LOOP_COUNTER };
}

impl UlpLoopCounter {
    pub const fn new(value: u32) -> Self {
        Self { 0: value }
    }

    pub fn count(&self) -> u32 {
        self.0
    }

    pub fn increment() {
        let mut c = Self::load();
        c.0 += 1;
        Self::store(c);
    }

    pub fn reset() {
        Self::store(0.into());
    }
}
