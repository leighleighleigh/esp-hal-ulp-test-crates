//! Increments a 32 bit counter value at a known point in memory, once a second.
//!
//! When using the ESP32-C6's LP core, this address in memory is `0x5000_2000`.
//!
//! When using the ESP32-S2 or ESP32-S3's ULP core, this address in memory is
//! `0x5000_0400` (but is `0x400`` from the ULP's point of view!).

#![no_std]
#![no_main]

use esp_lp_hal::prelude::*;
// use embedded_hal::delay::DelayNs;
// use esp_lp_hal::delay::Delay;
use panic_halt as _;

#[cfg(esp32c6)]
const ADDRESS: u32 = 0x5000_2000;

#[cfg(any(esp32s2, esp32s3))]
const ADDRESS: u32 = 0x1000;

#[entry]
fn main() {
    let ptr = ADDRESS as *mut u32;
    let mut i : u32 = unsafe { ptr.read_volatile() };
    i = i.wrapping_add(1u32);
    unsafe { ptr.write_volatile(i); }

    // Wake up the main CPU if the number is divisible by 10
    if i % 10 == 0 {
      esp_lp_hal::ulp_wake_hp_core();
    }
}
