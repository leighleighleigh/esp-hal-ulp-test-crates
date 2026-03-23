//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory, whenever the ULP program is run.
//! If GPIO0 is pressed, resets the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

use esp_lp_hal::{
    gpio::{Event, Input},
    gpio_interrupt,
    prelude::*,
    sens_interrupt,
};

const ADDRESS: u32 = 0x1000;

fn increment_counter() {
    let ptr = ADDRESS as *mut u32;
    let i = unsafe { ptr.read_volatile() };
    unsafe {
        ptr.write_volatile(i + 1);
    }
}

fn reset_counter() {
    let ptr = ADDRESS as *mut u32;
    unsafe {
        ptr.write_volatile(0);
    }
}

// Did we get a startup interrupt? If so, increment counter
sens_interrupt!(RISCV_START_INT, increment_counter);
// Reset counter on button press.
gpio_interrupt!(GPIO0, reset_counter);

// Hackily enable sens interrupt
#[allow(unused)]
fn enable_sens_intr() {
    unsafe { &*esp_lp_hal::pac::SENS::PTR }
        .sar_cocpu_int_ena()
        .write(|w| w.sar_cocpu_start_int_ena().set_bit());
}

#[allow(unused)]
fn disable_sens_intr() {
    unsafe { &*esp_lp_hal::pac::SENS::PTR }
        .sar_cocpu_int_ena()
        .write(|w| w.sar_cocpu_start_int_ena().clear_bit());
}

#[entry]
fn main(mut stomp_pin: Input<0>) {
    // Increment counter on start
    enable_sens_intr();
    // disable_sens_intr();
    // increment_counter();
    stomp_pin.listen(Event::FallingEdge);
    // stomp_pin.unlisten();
}
