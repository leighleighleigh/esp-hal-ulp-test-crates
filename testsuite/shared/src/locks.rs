#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

cfg_if::cfg_if! {
    if #[cfg(feature = "is-lp-core")] {
        #[unsafe(no_mangle)]
        #[used]
        pub static mut ULP_SHARED_LOCK: UlpLock = UlpLock::new();
    } else {
        unsafe extern "Rust" {
            pub static mut ULP_SHARED_LOCK: UlpLock;
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum LockTurn {
    Hp,
    Ulp,
}

// (Speculative) This needs to be 4-byte aligned so that the RISCV core can mutate single fields in
// a single instruction.
#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct UlpLock {
    pub flag_ulp: bool,
    pub flag_hp: bool,
    pub turn: LockTurn,
}

impl UlpLock {
    const fn new() -> Self {
        UlpLock {
            flag_ulp: false,
            flag_hp: false,
            turn: LockTurn::Hp,
        }
    }
}

// Based on
// https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/system/ulp-risc-v.html#_CPPv422ulp_riscv_lock_acquireP16ulp_riscv_lock_t
pub fn ulp_riscv_lock_acquire<T>(mut busy_loop_callback: T) -> Result<usize, usize>
where
    T: FnMut(usize) -> bool,
{
    // Count how many tries it took
    let mut tries = 0;

    cfg_if::cfg_if! {
        if #[cfg(feature = "is-lp-core")] {
        unsafe {
            ULP_SHARED_LOCK.flag_ulp = true;
            ULP_SHARED_LOCK.turn = LockTurn::Hp;

            while ULP_SHARED_LOCK.flag_hp && ULP_SHARED_LOCK.turn == LockTurn::Hp {
                tries += 1;
                // Call the busy loop callback,'
                // if it returns true then we will abort the waiting loop.
                if busy_loop_callback(tries) {
                    return Err(tries);
                }
            }
        }
        } else {
        unsafe {
            ULP_SHARED_LOCK.flag_hp = true;
            ULP_SHARED_LOCK.turn = LockTurn::Ulp;

            while ULP_SHARED_LOCK.flag_ulp && ULP_SHARED_LOCK.turn == LockTurn::Ulp {
                tries += 1;
                // Call the busy loop callback,
                // if it returns true then we will abort the waiting loop.
                if busy_loop_callback(tries) {
                    return Err(tries);
                }
            }
        }
        }
    }

    Ok(tries)
}

pub fn ulp_riscv_lock_release() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "is-lp-core")] {
            unsafe {
                ULP_SHARED_LOCK.flag_ulp = false;
            }
        } else {
            unsafe {
                ULP_SHARED_LOCK.flag_hp = false;
            }
        }
    }
}
