#![no_std]
#![no_main]
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]

cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
        // // SPACER VARIABLE
        // #[unsafe(no_mangle)]
        // #[used]
        // pub static mut __ULP_DUMMY__: u32 = 0;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_COMMAND: UlpCommandType = UlpCommandType::UNKNOWN;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_REPLY: UlpReplyType = UlpReplyType::UNKNOWN;

        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_LOOP_COUNTER: u32 = 0;
    } else {
        unsafe extern "Rust" {
            pub static mut ULP_COMMAND: UlpCommandType;
            pub static mut ULP_REPLY: UlpReplyType;
            pub static mut ULP_LOOP_COUNTER: u32;
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpCommandType {
    UNKNOWN            = 0,
    NOOP               = 1,
    LOOP_COUNTER_TEST  = 2,
    TIMER_COUNTER_TEST = 3,
    // RISCV_READ_WRITE_TEST = 1,
    // RISCV_DEEP_SLEEP_WAKEUP_SHORT_DELAY_TEST,
    // RISCV_DEEP_SLEEP_WAKEUP_LONG_DELAY_TEST,
    // RISCV_LIGHT_SLEEP_WAKEUP_TEST,
    // RISCV_STOP_TEST,
    // RISCV_MUTEX_TEST,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpReplyType {
    UNKNOWN       = 0,
    OK            = 1,
    NOK           = 2,
    UNIMPLEMENTED = 3,
}

pub struct UlpCommand {}

impl UlpCommand {
    #[inline(never)]
    pub fn read() -> UlpCommandType {
        // critical_section::with(|_cs| unsafe { ULP_COMMAND.clone() })
        unsafe { ULP_COMMAND.clone() }
        // unsafe {
        //     let p_mut = &mut ULP_COMMAND as *mut UlpCommandType;
        //     p_mut.read_unaligned()
        // }
    }

    #[inline(never)]
    pub fn write(value: UlpCommandType) {
        // critical_section::with(|_cs| unsafe {
        //     ULP_COMMAND = value;
        // })
        unsafe {
            ULP_COMMAND = value.clone();
        }
        // unsafe {
        //     let p_mut = &mut ULP_COMMAND as *mut UlpCommandType;
        //     p_mut.write_unaligned(value);
        // }
    }
}

pub struct UlpReply {}

impl UlpReply {
    #[inline(never)]
    pub fn read() -> UlpReplyType {
        // critical_section::with(|_cs| unsafe { ULP_REPLY.clone() })
        unsafe { ULP_REPLY.clone() }
        // unsafe {
        //     let p_mut = &mut ULP_REPLY as *mut UlpReplyType;
        //     p_mut.read_unaligned()
        // }
    }

    #[inline(never)]
    pub fn write(value: UlpReplyType) {
        // critical_section::with(|_cs| unsafe {
        //     ULP_REPLY = value;
        // })
        unsafe {
            ULP_REPLY = value.clone();
        }
        // unsafe {
        //     let p_mut = &mut ULP_REPLY as *mut UlpReplyType;
        //     p_mut.write_unaligned(value);
        // }
    }
}

pub struct UlpLoopCounter {}

impl UlpLoopCounter {
    #[inline(never)]
    pub fn read() -> u32 {
        // critical_section::with(|_cs| unsafe { ULP_LOOP_COUNTER.clone() })
        unsafe { ULP_LOOP_COUNTER.clone() }
        // unsafe {
        //     let p_mut: *mut u32 = &mut ULP_LOOP_COUNTER as *mut u32;
        //     p_mut.read_unaligned()
        // }
    }

    #[inline(never)]
    pub fn write(value: u32) {
        // critical_section::with(|_cs| unsafe {
        //     ULP_LOOP_COUNTER = value;
        // })
        unsafe { ULP_LOOP_COUNTER = value.clone() };
        // unsafe {
        //     let p_mut = &mut ULP_LOOP_COUNTER as *mut u32;
        //     p_mut.write_unaligned(value);
        // }
    }

    pub fn increment() {
        let c = Self::read();
        Self::write(c + 1);
    }

    pub fn reset() {
        Self::write(0);
    }
}
