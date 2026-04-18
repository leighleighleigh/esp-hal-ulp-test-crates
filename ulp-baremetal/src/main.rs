//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory, whenever the ULP program is run.
//! If GPIO0 is pressed, resets the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

use embedded_hal::digital::InputPin;
use esp_lp_hal::{
    gpio::{Input, Io},
    pac::Peripherals,
    prelude::*,
};

const ADDRESS: u32 = 0x1000;

// Only if interrupt supported
#[cfg(feature="interrupts")]
use core::cell::RefCell;
#[cfg(feature="interrupts")]
use critical_section::Mutex;
#[cfg(feature="interrupts")]
use esp_lp_hal::{interrupt::{self, Interrupt},gpio::Event};
#[cfg(feature="interrupts")]
static BUTTON: Mutex<RefCell<Option<Input<5>>>> = Mutex::new(RefCell::new(None));

#[inline]
fn counter_read() -> u32 {
    unsafe {
        let counter = ADDRESS as *mut u32;
        counter.read_volatile()
    }
}

#[inline]
fn counter_write(val: u32) {
    unsafe {
        let counter = ADDRESS as *mut u32;
        counter.write_volatile(val);
    }
}

#[entry]
fn main(mut button: Input<5>) {
    // Increment whenever woken up
    let c = counter_read();
    // Increment counter while button is pressed
    counter_write(c+1);

    // NOTE: Chaning the button listen / interrupt condition will affect GPIO wakeup.
    cfg_if::cfg_if! {
        if #[cfg(feature="interrupts")]
        {
            let peripherals = Peripherals::take().unwrap();
            let mut io = Io::new(peripherals.RTC_IO);
            io.set_interrupt_handler(gpio_interrupt_handler);
            critical_section::with(|cs| {
                button.listen(Event::FallingEdge,false);
                BUTTON.borrow_ref_mut(cs).replace(button);
            });
        } else {
          // Clear the GPIO wake-up flag
          esp_lp_hal::gpio_wakeup_clear();

          // Wakeup
          esp_lp_hal::wake_hp_core();

          // Debounce button
          let dly = esp_lp_hal::delay::Delay {};
          dly.delay_millis(500);

          // Re-set the wake-up flag for next iteration
          esp_lp_hal::gpio_wakeup_enable();
        }
    }
}

#[cfg(feature="interrupts")]
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
