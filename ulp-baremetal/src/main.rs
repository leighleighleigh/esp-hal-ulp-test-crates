//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory, whenever the ULP program is run.
//! If GPIO0 is pressed, resets the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

// use core::cell::RefCell;
// use critical_section::Mutex;

use esp_lp_hal::{
    gpio::{self, Event, Input, Io},
    interrupt,
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
fn reset_counter() {
    let ptr = ADDRESS as *mut u32;
    unsafe {
        ptr.write_volatile(0);
    }
}

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

// static BUTTON: Mutex<RefCell<Option<Input<0>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main(mut button: Input<0>) {
    // Get the io peripheral, and bind a handler to it.
    let peripherals = Peripherals::take();
    let mut _io = Io::new(peripherals.RTC_IO);
    _io.set_interrupt_handler(gpio_interrupt_handler);

    // TODO: Align the 'SENS' interrupt API with those exposed by the 'rt' flag of the PAC.
    //       See also ALT-TODO in interrupt.rs
    // critical_section::with(|_cs| {
    // interrupt::bind_handler(interrupt::Interrupt::SENS, sens_interrupt_handler);
    // enable_sens_intr();
    // });
    increment_counter();

    // critical_section::with(|cs| {
    //     button.listen(Event::FallingEdge);
    //     BUTTON.borrow_ref_mut(cs).replace(button)
    // });
    button.listen(Event::FallingEdge);
}

#[handler]
fn gpio_interrupt_handler() {
    // TODO: Create an enum for each GPIO pin? Maybe?
    let status = gpio::gpio_interrupt_status();
    if status & (0b1) != 0 {
        reset_counter();
    }
    gpio::gpio_interrupt_clear(status);

    // // Mirror the esp-hal API by passing the input pin through a mutex.
    // if critical_section::with(|cs| {
    //     BUTTON
    //         .borrow_ref_mut(cs)
    //         .as_mut()
    //         .unwrap()
    //         .is_interrupt_set()
    // }) {
    //     // esp_println::println!("Button was the source of the interrupt");
    //     reset_counter();
    // } else {
    //     // esp_println::println!("Button was not the source of the interrupt");
    // }

    // critical_section::with(|cs| {
    //     BUTTON
    //         .borrow_ref_mut(cs)
    //         .as_mut()
    //         .unwrap()
    //         .clear_interrupt()
    // });
}

#[handler]
fn sens_interrupt_handler() {
    // TODO read interrupt status in a more ergonomic manner.
    let was_riscv_start_int = unsafe { &*esp_lp_hal::pac::SENS::PTR }
        .sar_cocpu_int_st()
        .read()
        .sar_cocpu_start_int_st()
        .bit_is_set();
    if was_riscv_start_int {
        increment_counter();
    }
}
