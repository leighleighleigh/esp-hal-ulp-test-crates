#![no_std]
#![no_main]

#[cfg_attr(feature = "is-lp-core", unsafe(link_section = ".ulp"))]
#[cfg_attr(not(feature = "is-lp-core"),unsafe(link_section = ".ulp.ULP_SHARED_DATA"))]
#[unsafe(no_mangle)]
#[used]
pub static mut ULP_SHARED_DATA: u32 = 0xcafebabe;

// #[repr(C, align(2))]
// #[derive(Clone, Copy, Debug)]

// Mode setting, set by HP core, read by LP core
#[cfg(esp32c6)]
pub const COMMAND_ADDRESS: u32 = 0x5000_0804;
#[cfg(feature = "is-lp-core")]
#[cfg(any(esp32s2, esp32s3))]
pub const COMMAND_ADDRESS: u32 = 0x804;
#[cfg(not(feature = "is-lp-core"))]
#[cfg(any(esp32s2, esp32s3))]
pub const COMMAND_ADDRESS: u32 = 0x5000_0804;

// Reply , set by LP core, read by HP core
#[cfg(esp32c6)]
pub const REPLY_ADDRESS: u32 = 0x5000_0808;
#[cfg(feature = "is-lp-core")]
#[cfg(any(esp32s2, esp32s3))]
pub const REPLY_ADDRESS: u32 = 0x0808;
#[cfg(not(feature = "is-lp-core"))]
#[cfg(any(esp32s2, esp32s3))]
pub const REPLY_ADDRESS: u32 = 0x5000_0808;

// Counter, incremented by LP core, read by HP core
#[cfg(esp32c6)]
pub const COUNTER_ADDRESS: u32 = 0x5000_0800;
#[cfg(feature = "is-lp-core")]
#[cfg(any(esp32s2, esp32s3))]
pub const COUNTER_ADDRESS: u32 = 0x0800;
#[cfg(not(feature = "is-lp-core"))]
#[cfg(any(esp32s2, esp32s3))]
pub const COUNTER_ADDRESS: u32 = 0x5000_0800;

// #[inline]
pub fn reg_read(addr: u32) -> u32 {
    unsafe {
        let counter = addr as *mut u32;
        counter.read_unaligned()
    }
}

// #[inline]
pub fn reg_write(addr: u32, val: u32) {
    unsafe {
        let counter = addr as *mut u32;
        counter.write_unaligned(val);
    }
}

pub trait RW: Sized + PartialEq {
    const ADDR: u32;

    fn load() -> Self {
        Self::from_raw(reg_read(Self::ADDR))
    }

    fn save(self) {
        let v = self.into_raw();
        reg_write(Self::ADDR, v);
    }

    fn into_raw(&self) -> u32;
    fn from_raw(value: u32) -> Self;

    // Important for cross-arch compat
    fn eq_raw(&self, other: &Self) -> bool {
        self.into_raw() == other.into_raw()
    }
}


#[derive(Clone, Copy, defmt::Format)]
#[allow(non_camel_case_types)]
pub struct UlpLoopCounter {
    counter : u32
}

impl RW for UlpLoopCounter {
    const ADDR: u32 = COUNTER_ADDRESS;

    fn into_raw(&self) -> u32 {
        self.counter
    }

    fn from_raw(value: u32) -> Self {
        Self { counter: value }
    }
}

impl PartialEq for UlpLoopCounter {
    fn eq(&self, other: &Self) -> bool {
        self.eq_raw(other)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum UlpCommand {
    RISCV_COUNTER_TEST           = 0,
    RISCV_ULP_TIMER_COUNTER_TEST = 1,
    // RISCV_READ_WRITE_TEST = 1,
    // RISCV_DEEP_SLEEP_WAKEUP_SHORT_DELAY_TEST,
    // RISCV_DEEP_SLEEP_WAKEUP_LONG_DELAY_TEST,
    // RISCV_LIGHT_SLEEP_WAKEUP_TEST,
    // RISCV_STOP_TEST,
    // RISCV_MUTEX_TEST,
    RISCV_NO_COMMAND             = 2,
    RISCV_UNKNOWN_COMMAND(u32),
}

impl RW for UlpCommand {
    const ADDR: u32 = COMMAND_ADDRESS;
    fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::RISCV_COUNTER_TEST,
            1 => Self::RISCV_ULP_TIMER_COUNTER_TEST,
            2 => Self::RISCV_NO_COMMAND,
            v => Self::RISCV_UNKNOWN_COMMAND(v),
        }
    }
    fn into_raw(&self) -> u32 {
        match self {
            Self::RISCV_COUNTER_TEST => 0,
            Self::RISCV_ULP_TIMER_COUNTER_TEST => 1,
            Self::RISCV_NO_COMMAND => 2,
            Self::RISCV_UNKNOWN_COMMAND(v) => *v,
        }
    }
}

impl PartialEq for UlpCommand {
    fn eq(&self, other: &Self) -> bool {
        self.eq_raw(other)
    }
}

#[derive(Clone, Copy, defmt::Format)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum UlpReply {
    RISCV_COMMAND_OK            = 0,
    RISCV_COMMAND_NOK           = 1,
    RISCV_COMMAND_UNIMPLIMENTED = 2,
    RISCV_COMMAND_UNKNOWN(u32),
}

impl RW for UlpReply {
    const ADDR: u32 = REPLY_ADDRESS;
    fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::RISCV_COMMAND_OK,
            1 => Self::RISCV_COMMAND_NOK,
            2 => Self::RISCV_COMMAND_UNIMPLIMENTED,
            v => Self::RISCV_COMMAND_UNKNOWN(v),
        }
    }
    fn into_raw(&self) -> u32 {
        match self {
            Self::RISCV_COMMAND_OK => 0,
            Self::RISCV_COMMAND_NOK => 1,
            Self::RISCV_COMMAND_UNIMPLIMENTED => 2,
            Self::RISCV_COMMAND_UNKNOWN(va) => *va,
        }
    }
}

impl PartialEq for UlpReply {
    fn eq(&self, other: &Self) -> bool {
        self.eq_raw(other)
    }
}
