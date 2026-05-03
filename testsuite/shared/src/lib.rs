#![no_std]
#![no_main]

// Mode setting, set by HP core, read by LP core
#[cfg(esp32c6)]
pub const COMMAND_ADDRESS: u32 = 0x5000_1004;
#[cfg(feature = "is-lp-core")]
#[cfg(any(esp32s2, esp32s3))]
pub const COMMAND_ADDRESS: u32 = 0x1004;
#[cfg(not(feature = "is-lp-core"))]
#[cfg(any(esp32s2, esp32s3))]
pub const COMMAND_ADDRESS: u32 = 0x5000_1004;

// Reply , set by LP core, read by HP core
#[cfg(esp32c6)]
pub const REPLY_ADDRESS: u32 = 0x5000_1008;
#[cfg(feature = "is-lp-core")]
#[cfg(any(esp32s2, esp32s3))]
pub const REPLY_ADDRESS: u32 = 0x1008;
#[cfg(not(feature = "is-lp-core"))]
#[cfg(any(esp32s2, esp32s3))]
pub const REPLY_ADDRESS: u32 = 0x5000_1008;

// Counter, incremented by LP core, read by HP core
#[cfg(esp32c6)]
pub const COUNTER_ADDRESS: u32 = 0x5000_1000;
#[cfg(feature = "is-lp-core")]
#[cfg(any(esp32s2, esp32s3))]
pub const COUNTER_ADDRESS: u32 = 0x1000;
#[cfg(not(feature = "is-lp-core"))]
#[cfg(any(esp32s2, esp32s3))]
pub const COUNTER_ADDRESS: u32 = 0x5000_1000;

// #[inline]
pub fn reg_read(addr: u32) -> u32 {
    unsafe {
        let counter = addr as *mut u32;
        counter.read_volatile()
    }
}

// #[inline]
pub fn reg_write(addr: u32, val: u32) {
    unsafe {
        let counter = addr as *mut u32;
        counter.write_volatile(val);
    }
}

pub trait RW: Sized {
    const ADDR: u32;

    fn read() -> Self {
        Self::from_raw(reg_read(Self::ADDR))
    }

    fn write(&self) {
        let v = self.into_raw();
        reg_write(Self::ADDR, v);
    }

    fn into_raw(&self) -> u32;
    fn from_raw(value: u32) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum UlpCommand {
    RISCV_COUNTER_TEST           = 1,
    RISCV_ULP_TIMER_COUNTER_TEST = 2,
    // RISCV_READ_WRITE_TEST = 1,
    // RISCV_DEEP_SLEEP_WAKEUP_SHORT_DELAY_TEST,
    // RISCV_DEEP_SLEEP_WAKEUP_LONG_DELAY_TEST,
    // RISCV_LIGHT_SLEEP_WAKEUP_TEST,
    // RISCV_STOP_TEST,
    // RISCV_MUTEX_TEST,
    RISCV_NO_COMMAND             = 3,
    RISCV_UNKNOWN_COMMAND(u32),
}

impl RW for UlpCommand {
    const ADDR: u32 = COMMAND_ADDRESS;
    fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::RISCV_COUNTER_TEST,
            2 => Self::RISCV_ULP_TIMER_COUNTER_TEST,
            3 => Self::RISCV_NO_COMMAND,
            v => Self::RISCV_UNKNOWN_COMMAND(v),
        }
    }
    fn into_raw(&self) -> u32 {
        match self {
            Self::RISCV_COUNTER_TEST => 1,
            Self::RISCV_ULP_TIMER_COUNTER_TEST => 2,
            Self::RISCV_NO_COMMAND => 3,
            Self::RISCV_UNKNOWN_COMMAND(v) => *v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, defmt::Format)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum UlpReply {
    RISCV_COMMAND_OK            = 1,
    RISCV_COMMAND_NOK           = 2,
    RISCV_COMMAND_UNIMPLIMENTED = 3,
    RISCV_COMMAND_UNKNOWN(u32),
}

impl RW for UlpReply {
    const ADDR: u32 = REPLY_ADDRESS;
    fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::RISCV_COMMAND_OK,
            2 => Self::RISCV_COMMAND_NOK,
            3 => Self::RISCV_COMMAND_UNIMPLIMENTED,
            v => Self::RISCV_COMMAND_UNKNOWN(v),
        }
    }
    fn into_raw(&self) -> u32 {
        match self {
            Self::RISCV_COMMAND_OK => 1,
            Self::RISCV_COMMAND_NOK => 2,
            Self::RISCV_COMMAND_UNIMPLIMENTED => 3,
            Self::RISCV_COMMAND_UNKNOWN(va) => *va,
        }
    }
}
