#![no_std]
#![no_main]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]
// #![feature(associated_type_defaults)]

mod locks;

pub use locks::{UlpLock, ULP_LOCK};

pub const TEST_XOR_MASK: u32 = 0xcafe;
pub const TEST_MUTEX_ITERATIONS: u32 = 1000;

cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
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

        // This is actually a HP core variable,
        // although it's persistence is maintained using ULP-core memory.
        #[unsafe(no_mangle)]
        #[used]
        pub static mut HP_SLEEP_WAKEUP_COUNTER: u32 = 0;
        #[unsafe(no_mangle)]
        #[used]
        pub static mut HP_SLEEP_WAKEUP_CAUSE: u32 = 0;
    } else {
        unsafe extern "Rust" {
            pub static mut ULP_COMMAND: UlpCommand;
            pub static mut ULP_REPLY: UlpReply;
            pub static mut ULP_BOOT_COUNTER: u32;
            pub static mut ULP_HALT_COUNTER: u32;
            pub static mut ULP_LOOP_COUNTER: u32;
            pub static mut ULP_TEST_DATA_IN : u32;
            pub static mut ULP_TEST_DATA_OUT : u32;
            pub static mut HP_SLEEP_WAKEUP_COUNTER : u32;
            pub static mut HP_SLEEP_WAKEUP_CAUSE: u32;
        }
    }
}

// Trait for these unique shared variable types
pub trait SharedType: Sized {
    type VarType;
    const VAR: *const Self::VarType;

    // Load a value from memory
    fn load() -> Self::VarType {
        unsafe {
            let p = <Self as SharedType>::VAR;
            p.read_volatile()
        }
    }

    // Store a value to memory
    fn store(self)
    where
        Self: SharedType,
        Self: Into<<Self as SharedType>::VarType>,
    {
        unsafe {
            let ptr = <Self as SharedType>::VAR;
            let mut_ptr = ptr.cast_mut();
            mut_ptr.write_volatile(self.into());
        }
    }
}

pub trait SharedTypeConversion: SharedType
where
    Self: SharedType,
    Self: Into<<Self as SharedType>::VarType>,
    Self: From<<Self as SharedType>::VarType>,
{
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpCommand {
    UNSET             = 0,
    NOOP              = 1,
    COUNTER_ONESHOT   = 2,
    COUNTER_LOOP      = 3,
    COUNTER_ULP_TIMER = 4,
    XOR_TEST          = 5,
    STOP_TEST         = 6,
    MUTEX_TEST        = 7,
    TIMER_PERIOD_TEST = 8,
    LIGHT_SLEEP_TEST  = 9,
    // RISCV_DEEP_SLEEP_WAKEUP_SHORT_DELAY_TEST,
    // RISCV_DEEP_SLEEP_WAKEUP_LONG_DELAY_TEST,
    // RISCV_LIGHT_SLEEP_WAKEUP_TEST,
}

impl SharedType for UlpCommand {
    const VAR: *const Self = unsafe { &ULP_COMMAND };
    type VarType = Self;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpReply {
    UNSET = 0,
    OK    = 1,
    NOK   = 2,
}

impl SharedType for UlpReply {
    const VAR: *const Self = unsafe { &ULP_REPLY };
    type VarType = Self;
}

// Must be a struct so that we can make different trait impls for it.

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub struct UlpLoopCounter(u32);

impl From<u32> for UlpLoopCounter {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl Into<u32> for UlpLoopCounter {
    fn into(self) -> u32 {
        self.0
    }
}

impl SharedType for UlpLoopCounter {
    const VAR: *const Self::VarType = unsafe { &ULP_LOOP_COUNTER };
    type VarType = u32;
}

impl SharedTypeConversion for UlpLoopCounter {}

impl UlpLoopCounter {
    pub fn increment() {
        let c = Self::load();
        Self::store(UlpLoopCounter(c + 1));
    }

    pub fn reset() {
        Self::store(UlpLoopCounter(0));
    }
}

// BOOT COUNT
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub struct UlpBootCounter(u32);

impl From<u32> for UlpBootCounter {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl Into<u32> for UlpBootCounter {
    fn into(self) -> u32 {
        self.0
    }
}

impl SharedType for UlpBootCounter {
    const VAR: *const Self::VarType = unsafe { &ULP_BOOT_COUNTER };
    type VarType = u32;
}
impl SharedTypeConversion for UlpBootCounter {}

impl UlpBootCounter {
    pub fn increment() {
        let c = Self::load();
        Self::store(UlpBootCounter(c + 1));
    }

    pub fn reset() {
        Self::store(UlpBootCounter(0));
    }
}

// HALT COUNT (number of times the ULP's main function has returned.)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub struct UlpHaltCounter(u32);

impl From<u32> for UlpHaltCounter {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl Into<u32> for UlpHaltCounter {
    fn into(self) -> u32 {
        self.0
    }
}
impl SharedType for UlpHaltCounter {
    const VAR: *const Self::VarType = unsafe { &ULP_HALT_COUNTER };
    type VarType = u32;
}
impl SharedTypeConversion for UlpHaltCounter {}
impl UlpHaltCounter {
    pub fn increment() {
        let c = Self::load();
        Self::store(UlpHaltCounter(c + 1));
    }
    pub fn reset() {
        Self::store(UlpHaltCounter(0));
    }
}

// SLEEP WAKEUP COUNTER
// Incremented when the HP core wakes from a sleeping state,
// e.g. due to RTC timer, or LP-core interrupt.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub struct HpSleepWakeCounter(u32);

impl From<u32> for HpSleepWakeCounter {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl Into<u32> for HpSleepWakeCounter {
    fn into(self) -> u32 {
        self.0
    }
}
impl SharedType for HpSleepWakeCounter {
    const VAR: *const Self::VarType = unsafe { &HP_SLEEP_WAKEUP_COUNTER };
    type VarType = u32;
}
impl SharedTypeConversion for HpSleepWakeCounter {}

impl HpSleepWakeCounter {
    pub fn increment() {
        let c = Self::load();
        Self::store(HpSleepWakeCounter(c + 1));
    }
    pub fn reset() {
        Self::store(HpSleepWakeCounter(0));
    }
}

// HP core wakeup cause variable.
// To store the result of esp_hal::system::wakeup_cause(),
// between test executions. This is needed, because sleeping will break the test probe.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub struct HpSleepWakeupCause(u32);
impl From<u32> for HpSleepWakeupCause {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl Into<u32> for HpSleepWakeupCause {
    fn into(self) -> u32 {
        self.0
    }
}
impl SharedType for HpSleepWakeupCause {
    type VarType = u32;
    const VAR: *const Self::VarType = unsafe { &HP_SLEEP_WAKEUP_CAUSE };
}
impl SharedTypeConversion for HpSleepWakeupCause {}
