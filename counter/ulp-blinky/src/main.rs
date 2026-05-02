//! Increments a 32 bit counter value at a known point in memory, once a second.
//! If the 'mode' setting is 0, the chip will loop infinitely
//! If the 'mode' setting is 1, the chip will exit the infinite loop, and halt.
//! This allows ULP Timer functions, to be tested using the same LP-core firmware.

#![no_std]
#![no_main]

use esp_lp_hal::prelude::*;
use esp_lp_hal::delay::Delay;
use panic_halt as _;

// Mode setting, set by HP core, read by LP core
#[cfg(esp32c6)]
const MODE_ADDRESS: u32 = 0x5000_1004;
#[cfg(any(esp32s2, esp32s3))]
const MODE_ADDRESS: u32 = 0x1004;

// Counter, incremented by LP core, read by HP core
#[cfg(esp32c6)]
const COUNTER_ADDRESS: u32 = 0x5000_1000;
#[cfg(any(esp32s2, esp32s3))]
const COUNTER_ADDRESS: u32 = 0x1000;

#[inline]
pub fn reg_read(addr: u32) -> u32 {
    unsafe {
        let counter = addr as *mut u32;
        counter.read_volatile()
    }
}

#[inline]
pub fn reg_write(addr: u32, val: u32) {
    unsafe {
        let counter = addr as *mut u32;
        counter.write_volatile(val);
    }
}

#[entry]
fn main() {
    let dly = Delay {};
    let mode : u32 = reg_read(MODE_ADDRESS);
    let mut count : u32 = reg_read(COUNTER_ADDRESS);

    loop {
      count = count.wrapping_add(1u32);
      reg_write(COUNTER_ADDRESS, count);

      if mode == 1 {
        break;
      }

      dly.delay_millis(1000);
    }
}
