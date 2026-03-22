//! Increments a 32 bit counter value at a known point in memory, once a second.
//! Uses riscv_rt to provide interrupt handling, although it's linker scripts are overridden
//! with ones inside of ./ld/.

#![no_std]
#![no_main]

extern crate panic_halt;

use esp_lp_hal::prelude::*;
use esp_lp_hal::delay::Delay;
use esp_lp_hal::gpio::Input;
use esp_lp_hal::TrapFrame;
use esp_lp_hal::sens_interrupt;
use esp_lp_hal::gpio_interrupt;

const ADDRESS: u32 = 0x1000;

// Use the interrupt macro from esp-pacs with rt feature enabled, to hook an interrupt.
// Note that the interrupts are dispatched from esp-lp-hal.
pub fn on_start() {
    // Did we get a startup interrupt? If so, increment counter
    let ptr = ADDRESS as *mut u32;
    let i = unsafe { ptr.read_volatile() };
    unsafe { ptr.write_volatile(i + 1); }
}

sens_interrupt!(RISCV_START_INT, on_start);

// Hook a button press
pub fn on_button() {
    // Reset the counter
    let ptr = ADDRESS as *mut u32;
    unsafe { ptr.write_volatile(0); }
}

gpio_interrupt!(GPIO5,on_button);

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

// Hackily enable sens interrupt
fn enable_sens_intr() {
    unsafe { &*esp_lp_hal::pac::SENS::PTR }.sar_cocpu_int_ena().write(|w| w.sar_cocpu_start_int_ena().set_bit());
}

#[entry]
fn main(mut _stomp_pin : Input<5>) {
    enable_sens_intr();
    enable_gpio_intr();

    // let dly = Delay{};
    // Handle interupts for 3 seconds, then reset.
    // This should trigger the 'start up' interrupt from SensInterrupt,
    // which should reset the counter to 0>
    // dly.delay_millis(3000);
}
