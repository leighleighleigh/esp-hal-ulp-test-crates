#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// use core::arch;
// use embedded_hal::delay::DelayNs;
// use super::SharedType;

cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_LOCK: UlpLock = UlpLock::new();
    } else {
        unsafe extern "Rust" {
            pub static mut ULP_LOCK: UlpLock;
        }
    }
}

// (Speculative) This needs to be 4-byte aligned so that the RISCV core can mutate single fields in
// a single instruction.
#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct UlpLock {
    pub flag_ulp: bool,
    pub flag_hp: bool,
    pub is_ulp_turn: bool,
}

// impl SharedType for UlpLock {
//     const BACKING_VAR: *const Self = unsafe { &ULP_LOCK };
// }

impl UlpLock {
    #[allow(dead_code)]
    const fn new() -> Self {
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
pub fn ulp_riscv_lock_acquire() {
    cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
        unsafe {
            ULP_LOCK.flag_ulp = true;
            ULP_LOCK.is_ulp_turn = false;

            while ULP_LOCK.flag_hp && !ULP_LOCK.is_ulp_turn {
                // must have atleast one instruction of delay here.
                core::arch::asm!("nop");
            }
        }
    } else {

        #[allow(dead_code)]
        fn distract_hp_core()
        {
        }

        unsafe {
            ULP_LOCK.flag_hp = true;
            ULP_LOCK.is_ulp_turn = true;

            while ULP_LOCK.flag_ulp && ULP_LOCK.is_ulp_turn {
                // To avoid needing to import ESP-HAL crates,
                // this function is enough to ensure the HP core doesnt hog the lock.
                core::hint::black_box(distract_hp_core());
            }
        }
    }
    }
}

pub fn ulp_riscv_lock_release() {
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
}

// pub fn ulp_riscv_lock_acquire_delayed<D>(mut wait_delay: D)
// where
//     D: DelayNs,
// {
//     cfg_if::cfg_if! {
//         if #[cfg(feature = "is-lp-core")] {
//         unsafe {
//             ULP_LOCK.flag_ulp = true;
//             ULP_LOCK.is_ulp_turn = false;

//             while ULP_LOCK.flag_hp && !ULP_LOCK.is_ulp_turn {
//                 wait_delay.delay_ms(1);
//             }
//         }
//         } else {
//         unsafe {
//             ULP_LOCK.flag_hp = true;
//             ULP_LOCK.is_ulp_turn = true;

//             while ULP_LOCK.flag_ulp && ULP_LOCK.is_ulp_turn {
//                 // wait_delay.delay_ms(1);
//                 wait_delay.delay_us(1);
//             }
//         }
//         }
//     }
// }

// pub fn ulp_riscv_lock_acquire_with_callback<T>(mut busy_loop_callback: T) -> Result<usize, usize>
// where
//     T: FnMut(usize) -> bool,
// {
//     // Count how many tries it took
//     let mut tries = 0;

//     cfg_if::cfg_if! {
//         if #[cfg(feature = "is-lp-core")] {
//         unsafe {
//             ULP_LOCK.flag_ulp = true;
//             ULP_LOCK.is_ulp_turn = false;

//             while ULP_LOCK.flag_hp && !ULP_LOCK.is_ulp_turn {
//                 tries += 1;
//                 // Call the busy loop callback,'
//                 // if it returns true then we will abort the waiting loop.
//                 if busy_loop_callback(tries) {
//                     return Err(tries);
//                 }
//             }
//         }
//         } else {
//         unsafe {
//             ULP_LOCK.flag_hp = true;
//             ULP_LOCK.is_ulp_turn = true;

//             while ULP_LOCK.flag_ulp && ULP_LOCK.is_ulp_turn {
//                 tries += 1;
//                 // Call the busy loop callback,
//                 // if it returns true then we will abort the waiting loop.
//                 if busy_loop_callback(tries) {
//                     return Err(tries);
//                 }
//             }
//         }
//         }
//     }

//     Ok(tries)
// }
