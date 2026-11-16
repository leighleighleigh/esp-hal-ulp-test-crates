use core::cell::SyncUnsafeCell;

// Inner type, exists for interior mutability reasons
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct UlpLockData {
    pub flag_ulp: bool,
    pub flag_hp: bool,
    pub is_ulp_turn: bool,
}

impl UlpLockData {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self {
            flag_ulp: false,
            flag_hp: false,
            is_ulp_turn: false,
        }
    }
}

// (I think) This needs to be 4-byte aligned so that the
// RISCV core can mutate single fields in a single instruction.
// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[repr(C, align(4))]
pub struct UlpLockWrapper {
    pub inner: SyncUnsafeCell<UlpLockData>,
}

impl UlpLockWrapper {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        UlpLockWrapper {
            inner: SyncUnsafeCell::new(UlpLockData::new()),
        }
    }

    // Blocks until the lock is available.
    pub fn acquire(&self) {
        // Based on
        // https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/system/ulp-risc-v.html#_CPPv422ulp_riscv_lock_acquireP16ulp_riscv_lock_t
        cfg_if::cfg_if! {
            if #[cfg(feature = "is-lp-core")] {
                unsafe {
                    let x = self.inner.get().as_mut_unchecked();
                    x.flag_ulp = true;
                    x.is_ulp_turn = false;
                }

                loop {
                    unsafe {
                        let u = self.inner.get().as_mut_unchecked();
                        if u.flag_hp && !u.is_ulp_turn {
                            // must have atleast one instruction of delay here.
                            memory_fence_nop();
                        } else {
                            break;
                        }
                    }
                }
            } else {
                unsafe {
                    let x = self.inner.get().as_mut_unchecked();
                    x.flag_hp = true;
                    x.is_ulp_turn = true;
                }
                memory_fence_nop();

                loop {
                    unsafe {
                        let u = self.inner.get().as_mut_unchecked();
                        if u.flag_ulp && u.is_ulp_turn {
                            // must have atleast one instruction of delay here.
                            memory_fence_nop();
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn release(&self) {
        unsafe {
            let x = self.inner.get().as_mut_unchecked();
            cfg_if::cfg_if! {
                if #[cfg(feature = "is-lp-core")] {
                    x.flag_ulp = false;
                } else {
                    x.flag_hp = false;
                }
            }
        }
        memory_fence_nop();
    }

    pub fn reset(&self) {
        unsafe {
            let x = self.inner.get().as_mut_unchecked();
            x.flag_hp = false;
            x.flag_ulp = false;
            x.is_ulp_turn = false;
        }
    }
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
