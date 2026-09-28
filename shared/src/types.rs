use super::*;

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
    EXCEPTION_TEST    = 10,
    START_INT_TEST    = 11,
    GPIO_INT_TEST     = 12,
    GPIO_WAKEUP_TEST  = 13,
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
