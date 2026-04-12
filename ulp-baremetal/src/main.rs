//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory, whenever the ULP program is run.
//! If GPIO0 is pressed, resets the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

use core::cell::RefCell;

use critical_section::Mutex;
use esp_lp_hal::{
    gpio::{Event, Input, Io, WakeEvent},
    interrupt::{self, Interrupt},
    pac::Peripherals,
    prelude::*,
};

const ADDRESS: u32 = 0x1000;

static BUTTON: Mutex<RefCell<Option<Input<0>>>> = Mutex::new(RefCell::new(None));

#[inline]
fn counter_read() -> u32 {
    unsafe {
        let counter = ADDRESS as *mut u32;
        counter.read_volatile()
    }
}

#[inline]
fn counter_write(val : u32) {
    unsafe {
        let counter = ADDRESS as *mut u32;
        counter.write_volatile(val);
    }
}

#[entry]
fn main(mut button: Input<0>) {
    // Clear the GPIO wake-up flag
    esp_lp_hal::gpio::gpio_wakeup_clear();

    // Incriment counter
    counter_write(counter_read()+1);

    // Re-enable the wakeup bit
    esp_lp_hal::gpio::gpio_wakeup_enable(true);

    // let peripherals = Peripherals::take().unwrap();
    // let mut io = Io::new(peripherals.RTC_IO);
    // io.set_interrupt_handler(gpio_interrupt_handler);
    //critical_section::with(|cs| {
    //    button.listen(Event::FallingEdge);
    //    BUTTON.borrow_ref_mut(cs).replace(button);
    //});
    // interrupt::bind_handler(Interrupt::RISCV_START_INT, startup_interrupt_handler);
}

#[handler]
fn startup_interrupt_handler() {
    // Increment the counter every time RISCV_START_INT is triggered
    counter_write(counter_read() + 1);

    // On entry, immediately disable any more start-up interrupts.
    // This is needed, because RISCV_START_INT is driven from the ULP Timer,
    // which may be called multiple times before main() has finished execution.
    interrupt::set_enabled(Interrupt::RISCV_START_INT,false);
}

#[handler]
fn gpio_interrupt_handler() {
    // Check if BUTTON has an interrupt pending
    if critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        // The button was the source of the interrupt, reset the counter to 0.
        counter_write(0);
    }

    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
