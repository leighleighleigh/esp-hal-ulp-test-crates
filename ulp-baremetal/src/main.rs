//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory, whenever the ULP program is run.
//! If GPIO0 is pressed, resets the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

use embedded_hal::digital::InputPin;
use esp_lp_hal::{
    gpio::{Event, Input, Io, WakeEvent},
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
use esp_lp_hal::interrupt::{self, Interrupt};
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

    // // NOTE: Chaning the button listen / interrupt condition will affect GPIO wakeup.
    // cfg_if::cfg_if! {
    //     if #[cfg(feature="interrupts")]
    //     {
    //         let peripherals = Peripherals::take().unwrap();
    //         let mut io = Io::new(peripherals.RTC_IO);
    //         io.set_interrupt_handler(gpio_interrupt_handler);
    //         critical_section::with(|cs| {
    //             button.listen(Event::HighLevel);
    //             BUTTON.borrow_ref_mut(cs).replace(button);
    //         });
    //     }
    // }

    // // Increment whenever woken up
    // let c = counter_read();
    // // Increment counter while button is pressed
    // counter_write(c+1);

    // Wakeup
    //esp_lp_hal::wake_hp_core();
    alt_wake_hp_core();

    // Debounce
    let dly = esp_lp_hal::delay::Delay {};
    dly.delay_millis(1000);

    // Clear the GPIO wake-up flag
    //esp_lp_hal::gpio::gpio_wakeup_clear();
    alt_gpio_wakeup_clear();
}

fn alt_wake_hp_core() {
  // 0000004e <ulp_riscv_wakeup_main_processor>:
  // 4e:   67a1                    lui     a5,0x8
  // 50:   4f98                    lw      a4,24(a5)
  // 52:   00176713                ori     a4,a4,1
  // 56:   cf98                    sw      a4,24(a5)
  // 58:   8082                    ret
  unsafe {
    let temp_a5 : i32;
    let temp_a4 : i32;

    core::arch::asm!(
      "lui {0}, 0x8",
      "lw {1}, 24({0})",
      "ori {1}, {1}, 1",
      "sw {1}, 24({0})",
      out(reg) temp_a5,
      out(reg) temp_a4,
    );
  }
}

fn alt_gpio_wakeup_clear() {
  // 0000007e <ulp_riscv_gpio_wakeup_clear>:
  // 7e:   67a1                    lui     a5,0x8
  // 80:   0fc78793                addi    a5,a5,252 # 80fc <RTCCNTL+0xfc>
  // 84:   4398                    lw      a4,0(a5)
  // 86:   400006b7                lui     a3,0x40000
  // 8a:   8f55                    or      a4,a4,a3
  // 8c:   c398                    sw      a4,0(a5)
  // 8e:   8082                    ret
  unsafe {
    let temp_a5 : i32;
    let temp_a4 : i32;
    let temp_a3 : i32;

    core::arch::asm!(
      "lui {0}, 0x8",
      "addi {0}, {0}, 252",
      "lw {1}, 0({0})",
      "lui {2}, 0x40000",
      "or {1},{1},{2}",
      "sw {1},0({0})",
      out(reg) temp_a5,
      out(reg) temp_a4,
      out(reg) temp_a3,
    );
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
      //  counter_write(0);

      // Wakeup the HP core
      alt_wake_hp_core();
   }
   critical_section::with(|cs| {
       BUTTON
           .borrow_ref_mut(cs)
           .as_mut()
           .unwrap()
           .clear_interrupt()
   });
}
