#![allow(unused)]
// Trait for these unique shared variable types
pub trait SharedType: Sized {
    type VarType;
    const VAR: *const Self::VarType;

    // Load a value from memory
    fn load() -> Self::VarType {
        unsafe {
            let p = <Self as SharedType>::VAR;
            p.read_volatile()
        }
    }

    // Store a value to memory
    fn store(self)
    where
        Self: SharedType,
        Self: Into<<Self as SharedType>::VarType>,
    {
        unsafe {
            let ptr = <Self as SharedType>::VAR;
            let mut_ptr = ptr.cast_mut();
            mut_ptr.write_volatile(self.into());
        }
    }

    // Store an instance of VarType to memory
    fn store_var(value: Self::VarType) {
        unsafe {
            let ptr = Self::VAR;
            let mut_ptr = ptr.cast_mut();
            mut_ptr.write_volatile(value);
        }
    }

    // Obtain a mutable reference to the backing variable
    fn load_mut_ref<'a>() -> &'a mut Self::VarType {
        unsafe { Self::VAR.cast_mut().as_mut_unchecked() }
    }
}

// Trait to conveniently convert between a shared variable and it's primitive backing type
pub trait SharedTypeConversion: SharedType
where
    Self: SharedType,
    Self: Into<<Self as SharedType>::VarType>,
    Self: From<<Self as SharedType>::VarType>,
{
}

// New trait that might replace SharedType
pub trait SharedVar<'singleton, InnerType>
where
    InnerType: Sized + 'static,
{
    // shared_mut_ptr is the only method that needs to be implemented
    // by the singleton struct. It should return a raw mut pointer to a mutable static
    // variable of type 'InnerType'.
    unsafe fn shared_mut_ptr() -> *mut InnerType;

    unsafe fn shared_ref_ptr() -> *const InnerType {
        unsafe { Self::shared_ref_ptr().as_ref_unchecked() }
    }

    // Shared references (mutable or not) will have the same lifetime as the singleton.

    fn shared_ref() -> &'singleton InnerType {
        unsafe { &*Self::shared_ref_ptr() }
    }

    fn shared_mut() -> &'singleton mut InnerType {
        unsafe { &mut *Self::shared_mut_ptr() }
    }
}

// use crate::UlpLockData;
// This trait can only be applied to things that implement
// SharedVar<', UlpLockData>, i.e. they wrap a UlpLockData type variable.
// pub trait SharedLock<'a>
// where
//     Self: SharedVar<'a, UlpLockData>,
// {
//     fn acquire() {
//         let ptr = Self::shared_mut();
//         core::hint::black_box(ptr.acquire_mut());
//     }
//     fn release() {
//         let ptr = Self::shared_mut();
//         core::hint::black_box(ptr.release_mut());
//     }
//     fn reset() {
//         let ptr = Self::shared_mut();
//         core::hint::black_box(ptr.reset_mut());
//     }
// }
