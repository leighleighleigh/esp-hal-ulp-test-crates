#![no_std]
#![no_main]

// This library is used by both LP and HP cores.
// It contains the shared global variables!

cfg_if::cfg_if! {
if #[cfg(feature = "is-lp-core")] {
    use esp_hal_procmacros::shared_variable;
    #[shared_variable]
    pub static mut RAINBOW_COUNTER: u32 = 0;
    #[shared_variable]
    pub static mut RAINBOW_RUN: u32 = 0;
    #[shared_variable]
    pub static mut OTHER_VAR: u32 = 0;
} else {
    // TODO: Use a proc-macro to auto-generate these extern definitions.
    unsafe extern "Rust" {
        pub safe static mut RAINBOW_COUNTER: u32;
        pub safe static mut RAINBOW_RUN: u32;
        pub safe static mut OTHER_VAR: u32;
    }
}
}
