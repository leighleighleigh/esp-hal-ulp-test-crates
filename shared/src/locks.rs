use crate::traits::SharedCell;

// Inner type, exists for interior mutability reasons
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[repr(C, align(4))]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct UlpLockInner {
    pub flag_ulp: bool,
    pub flag_hp: bool,
    pub is_ulp_turn: bool,
}

impl UlpLockInner {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self {
            flag_ulp: false,
            flag_hp: false,
            is_ulp_turn: false,
        }
    }
}

pub type UlpLockWrapper = SharedCell<UlpLockInner>;

// Based on: https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/system/ulp-risc-v.html#_CPPv422ulp_riscv_lock_acquireP16ulp_riscv_lock_t
impl UlpLockWrapper {
    // Create a new lock
    pub const fn new() -> Self {
        Self::init(UlpLockInner::new())
    }

    // Blocks until the lock is available.
    #[cfg(feature = "is-lp-core")]
    pub fn acquire(&self) {
        let x = self.load_mut();
        x.flag_ulp = true;
        x.is_ulp_turn = false;

        loop {
            let u = self.load_mut();
            if u.flag_hp && !u.is_ulp_turn {
                // must have atleast one instruction of delay here.
                memory_fence_nop();
            } else {
                break;
            }
        }
    }

    // Blocks until the lock is available.
    #[cfg(not(feature = "is-lp-core"))]
    pub fn acquire(&self) {
        let x = self.load_mut();
        x.flag_hp = true;
        x.is_ulp_turn = true;
        memory_fence_nop();

        loop {
            let u = self.load_mut();
            if u.flag_ulp && u.is_ulp_turn {
                // must have atleast one instruction of delay here.
                memory_fence_nop();
            } else {
                break;
            }
        }
    }

    // Releases the lock, if held
    #[cfg(feature = "is-lp-core")]
    pub fn release(&self) {
        let x = self.load_mut();
        x.flag_ulp = false;
        memory_fence_nop();
    }

    // Releases the lock, if held
    #[cfg(not(feature = "is-lp-core"))]
    pub fn release(&self) {
        let x = self.load_mut();
        x.flag_hp = false;
        memory_fence_nop();
    }

    // Reset the lock state (unsafe)
    pub fn reset(&self) {
        let x = self.load_mut();
        x.flag_hp = false;
        x.flag_ulp = false;
        x.is_ulp_turn = false;
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
