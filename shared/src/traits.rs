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
}

// Trait to conveniently convert between a shared variable and it's primitive backing type
pub trait SharedTypeConversion: SharedType
where
    Self: SharedType,
    Self: Into<<Self as SharedType>::VarType>,
    Self: From<<Self as SharedType>::VarType>,
{
}
