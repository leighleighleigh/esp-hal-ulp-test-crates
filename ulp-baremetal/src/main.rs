//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory,
//! whenever the ULP program is run. If GPIO0 is pressed, reset the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

// TODO: I'd prefer if these were proc-macros, which could be used as function attributes.
//       This would 1) look nicer, and 2) provide better discoverability/type-hinting of the
// interrupts available for use.
use esp_lp_hal::{
    gpio::{Event, Input},
    gpio_interrupt,
    prelude::*,
    sens_interrupt,
};

const ADDRESS: u32 = 0x1000;

// Use the interrupt macro from esp-pacs with rt feature enabled, to hook an interrupt.
// Note that the interrupts are dispatched from esp-lp-hal.
pub fn on_start() {
    // Did we get a startup interrupt? If so, increment counter
    let ptr = ADDRESS as *mut u32;
    let i = unsafe { ptr.read_volatile() };
    unsafe {
        ptr.write_volatile(i + 1);
    }
}

sens_interrupt!(RISCV_START_INT, on_start);

// Hook a button press
pub fn on_button() {
    // Reset the counter
    let ptr = ADDRESS as *mut u32;
    unsafe {
        ptr.write_volatile(0);
    }
}

gpio_interrupt!(GPIO0, on_button);

// Hackily enable sens interrupt
fn enable_sens_intr() {
    unsafe { &*esp_lp_hal::pac::SENS::PTR }
        .sar_cocpu_int_ena()
        .write(|w| w.sar_cocpu_start_int_ena().set_bit());
}

#[entry]
fn main(mut stomp_pin: Input<0>) {
    enable_sens_intr();
    stomp_pin.listen(Event::FallingEdge);
}
