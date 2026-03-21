//! Increments a 32 bit counter value at a known point in memory, once a second.
//! Uses riscv_rt to provide interrupt handling, although it's linker scripts are overridden
//! with ones inside of ./ld/.

#![no_std]
#![no_main]

extern crate panic_halt;

use esp_lp_hal::prelude::*;
use esp_lp_hal::delay::Delay;
use esp_lp_hal::gpio::Input;
// use embedded_hal::digital::InputPin;
// use esp_lp_hal::TrapFrame;

const ADDRESS: u32 = 0x1000;

/// Default interrupt handler for RTC Peripheral interrupts (SENS).
/// Users may override SensInterrupt to change this behaviour.
#[unsafe(export_name = "GpioInterrupt")]
pub fn gpio_interrupt(_pin_status : u32) {
    let ptr = ADDRESS as *mut u32;
    // unsafe { ptr.write_volatile(pin_status) };
    let mut i : u32 = unsafe { ptr.read_volatile() };
    i = i.wrapping_add(1u32);
    unsafe { ptr.write_volatile(i); }
}

// Hackily enable a GPIO interrupt 
// Call this in your main()
fn enable_gpio_intr() {
    // RTC_GPIO_PINn_PAD_DRIVER (R/W)
    //   Pin driver selection.
    //   0: normal output
    //   1: open drain.

    // RTC_GPIO_PINn_INT_TYPE (R/W)
    // GPIO interrupt type selection.
    //   0: GPIO interrupt disabled
    //   1: rising edge trigger
    //   2: falling edge trigger
    //   3: any edge trigger
    //   4: low level trigger
    //   5: high level trigger.
    let io_int_type : u8 = 2;

    // RTC_GPIO_PINn_WAKEUP_ENABLE (R/W)
    //   GPIO wake-up enable. This will only wake up the chip from Light-sleep.
    unsafe { &*esp_lp_hal::pac::RTC_IO::PTR }.pin5().write(|w| unsafe { w.int_type().bits(io_int_type) });
}

#[entry]
fn main(mut _stomp_pin : Input<5>) {
    enable_gpio_intr();

    let dly = Delay{};
    // let ptr = ADDRESS as *mut u32;

    loop {
        // if stomp_pin.is_low().unwrap() {
        //     let mut i : u32 = unsafe { ptr.read_volatile() };
        //     i = i.wrapping_add(1u32);
        //     unsafe { ptr.write_volatile(i); }
        // }
        dly.delay_millis(100);
    }
}
