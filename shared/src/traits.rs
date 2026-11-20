#![allow(unused)]
use core::{
    cell::{RefMut, SyncUnsafeCell},
    ops::{Deref, DerefMut, RemAssign, Shl},
};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[repr(C, align(4))]
pub struct SharedCell<ET: Sized>
where
    ET: Clone + Copy + PartialEq + PartialOrd,
{
    inner: SyncUnsafeCell<ET>,
}

// TODO: Employ the delegate! macro somehow, to make type access more ergonomic?
impl<ET> SharedCell<ET>
where
    ET: Clone + Copy + PartialEq + PartialOrd,
{
    pub const fn init(init_value: ET) -> Self {
        Self {
            inner: SyncUnsafeCell::new(init_value),
        }
    }

    pub fn load(&self) -> ET {
        unsafe { self.inner.get().read_volatile() }
    }

    pub fn store(&self, value: ET) {
        unsafe {
            self.inner.get().replace(value);
        }
    }

    pub fn load_mut(&self) -> &mut ET {
        unsafe { self.inner.get().as_mut_unchecked() }
    }

    pub fn load_ref(&self) -> &ET {
        unsafe { self.inner.get().as_ref_unchecked() }
    }
}
