#![no_std]
#![no_main]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]

#[cfg_attr(feature = "is-lp-core", unsafe(link_section = ".ulp"))]
#[cfg_attr(
    not(feature = "is-lp-core"),
    unsafe(link_section = ".ulp.ULP_LOOP_COUNTER")
)]
#[unsafe(no_mangle)]
pub static mut ULP_LOOP_COUNTER: u32 = 0;

#[cfg_attr(feature = "is-lp-core", unsafe(link_section = ".ulp"))]
#[cfg_attr(not(feature = "is-lp-core"), unsafe(link_section = ".ulp.ULP_COMMAND"))]
#[unsafe(no_mangle)]
pub static mut ULP_COMMAND: UlpCommandType = UlpCommandType::RISCV_NO_COMMAND;

#[cfg_attr(feature = "is-lp-core", unsafe(link_section = ".ulp"))]
#[cfg_attr(not(feature = "is-lp-core"), unsafe(link_section = ".ulp.ULP_REPLY"))]
#[unsafe(no_mangle)]
pub static mut ULP_REPLY: UlpReplyType = UlpReplyType::RISCV_COMMAND_OK;

#[repr(C, align(2))]
#[derive(Clone, Copy, PartialEq, Debug, defmt::Format)]
pub enum UlpCommandType {
    RISCV_UNKNOWN_COMMAND = 0,
    RISCV_NO_COMMAND,
    RISCV_COUNTER_TEST,
    RISCV_ULP_TIMER_COUNTER_TEST,
    // RISCV_READ_WRITE_TEST = 1,
    // RISCV_DEEP_SLEEP_WAKEUP_SHORT_DELAY_TEST,
    // RISCV_DEEP_SLEEP_WAKEUP_LONG_DELAY_TEST,
    // RISCV_LIGHT_SLEEP_WAKEUP_TEST,
    // RISCV_STOP_TEST,
    // RISCV_MUTEX_TEST,
}

#[repr(C, align(2))]
#[derive(Clone, Copy, PartialEq, Debug, defmt::Format)]
pub enum UlpReplyType {
    RISCV_COMMAND_UNKNOWN = 0,
    RISCV_COMMAND_OK,
    RISCV_COMMAND_NOK,
    RISCV_COMMAND_UNIMPLIMENTED,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct UlpCommand {}

impl UlpCommand {
    pub fn read() -> UlpCommandType {
        critical_section::with(|_cs| unsafe { ULP_COMMAND.clone() })
    }

    pub fn write(value: UlpCommandType) {
        critical_section::with(|_cs| unsafe {
            ULP_COMMAND = value;
        })
    }
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct UlpReply {}

impl UlpReply {
    pub fn read() -> UlpReplyType {
        critical_section::with(|_cs| unsafe { ULP_REPLY.clone() })
    }

    pub fn write(value: UlpReplyType) {
        critical_section::with(|_cs| unsafe {
            ULP_REPLY = value;
        })
    }
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct UlpLoopCounter {}

impl UlpLoopCounter {
    pub fn read() -> u32 {
        critical_section::with(|_cs| unsafe { ULP_LOOP_COUNTER.clone() })
    }

    pub fn write(value: u32) {
        critical_section::with(|_cs| unsafe {
            ULP_LOOP_COUNTER = value;
        })
    }

    pub fn increment() {
        let c = Self::read() + 1;
        Self::write(c);
    }

    pub fn reset() {
        Self::write(0);
    }
}
