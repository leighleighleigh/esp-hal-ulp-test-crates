//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory, whenever the ULP program is run.
//! If GPIO0 is pressed, resets the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

use esp_lp_hal::{
    gpio::{Input, Io},
    pac::Peripherals,
    prelude::*,
};

const ADDRESS: u32 = 0x1000;

fn increment_counter() {
    let ptr = ADDRESS as *mut u32;
    let i = unsafe { ptr.read_volatile() };
    unsafe {
        ptr.write_volatile(i + 1);
    }
}

#[entry]
fn main(mut _button: Input<0>) {
    let peripherals = Peripherals::take().unwrap();
    let mut _io = Io::new(peripherals.RTC_IO);

    increment_counter();
}
