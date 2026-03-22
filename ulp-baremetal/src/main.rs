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

/// Override interrupt handler for RTC Peripheral interrupts (SENS).
#[unsafe(export_name = "SensInterrupt")]
pub fn sens_interrupt(sar_cocpu_int_st: esp_lp_hal::pac::sens::sar_cocpu_int_st::R) {
    // Did we get a startup interrupt? If so, reset counter to 0x0
    if sar_cocpu_int_st.sar_cocpu_start_int_st().bit_is_set() {
        let ptr = ADDRESS as *mut u32;
        unsafe { ptr.write_volatile(0); }
    }
}

/// Overriden handler for GPIO interrupts
#[unsafe(export_name = "GpioInterrupt")]
pub fn gpio_interrupt(pin_status : u32) {
    // Will increment counter when GPIO 5 is pressed
    let pin_number = 5;
    if (pin_status & (1 << pin_number)) != 0 {
        let ptr = ADDRESS as *mut u32;
        let mut i : u32 = unsafe { ptr.read_volatile() };
        i = i.wrapping_add(1u32);
        unsafe { ptr.write_volatile(i); }
    }
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

// Hackily enable sens interrupt
fn enable_sens_intr() {
    unsafe { &*esp_lp_hal::pac::SENS::PTR }.sar_cocpu_int_ena().write(|w| w.sar_cocpu_start_int_ena().set_bit());
}

#[entry]
fn main(mut _stomp_pin : Input<5>) {
    enable_sens_intr();
    enable_gpio_intr();

    let dly = Delay{};

    // Handle interupts for 3 seconds, then reset.
    // This should trigger the 'start up' interrupt from SensInterrupt,
    // which should reset the counter to 0>
    dly.delay_millis(3000);
}
