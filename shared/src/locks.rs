use super::*;

// (Speculative) This needs to be 4-byte aligned so that the
// RISCV core can mutate single fields in a single instruction.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct UlpLock {
    pub flag_ulp: bool,
    pub flag_hp: bool,
    pub is_ulp_turn: bool,
}

// impl SharedType for UlpLock {
//     const VAR: *const Self::VarType = unsafe { &ULP_LOCK };
//     type VarType = Self;
// }

impl UlpLock {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        UlpLock {
            flag_ulp: false,
            flag_hp: false,
            is_ulp_turn: false,
        }
    }

    // Blocks until the lock is available.
    pub fn acquire() {
        ulp_riscv_lock_acquire();
    }

    pub fn release() {
        ulp_riscv_lock_release();
    }

    pub fn reset() {
        unsafe {
            ULP_LOCK.flag_hp = false;
            ULP_LOCK.flag_ulp = false;
            ULP_LOCK.is_ulp_turn = false;
        }
    }
}

// Based on
// https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/system/ulp-risc-v.html#_CPPv422ulp_riscv_lock_acquireP16ulp_riscv_lock_t
fn ulp_riscv_lock_acquire() {
    cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
        unsafe {
            ULP_LOCK.flag_ulp = true;
            ULP_LOCK.is_ulp_turn = false;

            while ULP_LOCK.flag_hp && !ULP_LOCK.is_ulp_turn {
                // must have atleast one instruction of delay here.
                memory_fence_nop();
            }
        }
    } else {
        unsafe {
            ULP_LOCK.flag_hp = true;
            ULP_LOCK.is_ulp_turn = true;
            memory_fence_nop();

            while ULP_LOCK.flag_ulp && ULP_LOCK.is_ulp_turn {
                // must have atleast one instruction of delay here.
                memory_fence_nop();
            }
        }
    }
    }
}

fn ulp_riscv_lock_release() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "is-lp-core")] {
            unsafe {
                ULP_LOCK.flag_ulp = false;
            }
        } else {
            unsafe {
                ULP_LOCK.flag_hp = false;
            }
        }
    }
    memory_fence_nop();
}

// Ensures memory writes are not cached during lock manipulation
#[inline(always)]
fn memory_fence_nop() {
    #[cfg(target_arch = "xtensa")]
    unsafe {
        core::arch::asm!("memw");
        // core::arch::asm!("nop");
    }

    #[cfg(target_arch = "riscv32")]
    unsafe {
        #[cfg(not(feature = "is-lp-core"))]
        core::arch::asm!("fence");
        // LP/ULP core does not support fence instruction,
        // so will just no-op instead.
        #[cfg(feature = "is-lp-core")]
        core::arch::asm!("nop");
    }
}
