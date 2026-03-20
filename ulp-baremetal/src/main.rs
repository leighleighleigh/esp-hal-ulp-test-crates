//! Increments a 32 bit counter value at a known point in memory, once a second.
//! Uses riscv_rt to provide interrupt handling, although it's linker scripts are overridden
//! with ones inside of ./ld/.

#![no_std]
#![no_main]

extern crate panic_halt;
use esp_hal_procmacros::entry;

const ADDRESS: u32 = 0x1000;

#[entry]
fn main() -> ! {
    let ptr = ADDRESS as *mut u32;
    let mut i : u32 = unsafe { ptr.read_volatile() };
    i = i.wrapping_add(1u32);
    unsafe { ptr.write_volatile(i); }

    loop {}
}
