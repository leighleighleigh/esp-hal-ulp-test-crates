//! ULP interrupt-based counter example.
//! Increments a 32 bit counter value at a known point in memory, whenever the ULP program is run.
//! If GPIO0 is pressed, resets the counter.

#![no_std]
#![no_main]

extern crate panic_halt;

use esp_lp_hal::{
    gpio::{Input, Io},
    pac::Peripherals,
    prelude::*,
};

#[cfg(any(esp32s3, esp32s2))]
const ADDRESS: u32 = 0x1000;
#[cfg(esp32c6)]
const ADDRESS: u32 = 0x5000_1000;

#[cfg(any(esp32s3, esp32s2))]
const DEBUG_ADDRESS: u32 = 0x1004;
#[cfg(esp32c6)]
const DEBUG_ADDRESS: u32 = 0x5000_1004;

// Only if interrupt supported
cfg_if::cfg_if! {
    if #[cfg(feature = "gpio-interrupt")]
    {
        use core::cell::RefCell;
        use critical_section::Mutex;
        use esp_lp_hal::{
            gpio::Event,
            interrupt,
        };
        static BUTTON: Mutex<RefCell<Option<Input<5>>>> = Mutex::new(RefCell::new(None));
    }
}

#[inline]
fn reg_read(addr : u32) -> u32 {
    unsafe {
        let counter = addr as *mut u32;
        counter.read_volatile()
    }
}

#[inline]
fn reg_write(addr : u32, val: u32) {
    unsafe {
        let counter = addr as *mut u32;
        counter.write_volatile(val);
    }
}

#[entry]
fn main(mut button: Input<5>) {
    let dly = esp_lp_hal::delay::Delay {};

    #[cfg(feature = "ulp-timer")]
    #[cfg(feature = "gpio-wakeup")]
    {
        compile_error!("Cannot use both ulp-timer and gpio-wakeup features together!");
    }

    #[cfg(feature = "gpio-interrupt")]
    #[cfg(feature = "gpio-wakeup")]
    {
        compile_error!("Cannot use both gpio-interrupt and gpio-wakeup features together!");
    }

    // NOTE: Chaning the button listen / interrupt condition will affect GPIO wakeup.
    #[cfg(feature = "gpio-interrupt")]
    {
        let peripherals = Peripherals::take().unwrap();

        #[cfg(not(esp32c6))]
        let mut io = Io::new(peripherals.RTC_IO);

        #[cfg(esp32c6)]
        let mut io = Io::new(peripherals.LP_IO);

        io.set_interrupt_handler(gpio_interrupt_handler);
        interrupt::bind_handler(interrupt::Interrupt::RISCV_START_INT, startup_interrupt_handler);
        interrupt::set_enabled(interrupt::Interrupt::GPIO_INT, true);
        interrupt::set_enabled(interrupt::Interrupt::RISCV_START_INT, true);

        // critical_section::with(|cs| {
        //     button.listen(Event::RisingEdge, false);
        //     BUTTON.borrow_ref_mut(cs).replace(button);
        // });

        dly.delay_millis(1);
    }

    #[cfg(feature = "gpio-wakeup")]
    {
        // Clear the GPIO wake-up flag
        esp_lp_hal::gpio_wakeup_clear();
        // Wakeup
        esp_lp_hal::wake_hp_core();
        // Debounce button
        dly.delay_millis(1000);
        // Re-set the wake-up flag for next iteration
        // esp_lp_hal::gpio_wakeup_enable();
    }

    loop {
        // Increment whenever woken up
        let c = reg_read(ADDRESS);
        reg_write(ADDRESS, c + 1);
        // If ulp timer is enabled, the main loop will break.
        #[cfg(feature = "ulp-timer")]
        break;
        // else, a small delay so it runs at 10Hz
        #[cfg(not(feature = "ulp-timer"))]
        dly.delay_millis(100);
    }
}

#[cfg(feature = "gpio-interrupt")]
#[handler]
fn startup_interrupt_handler() {
    let c = reg_read(DEBUG_ADDRESS);
    reg_write(DEBUG_ADDRESS, c + 1);
    
    // must disable it to trigger only once
    interrupt::set_enabled(interrupt::Interrupt::RISCV_START_INT, false);
}

#[cfg(feature = "gpio-interrupt")]
#[handler]
fn gpio_interrupt_handler() {
    // reg_write(DEBUG_ADDRESS, esp_lp_hal::gpio::gpio_interrupt_status());
    // interrupt::clear(interrupt::Interrupt::GPIO_INT);

    // Check if BUTTON has an interrupt pending
    if critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        // The button was the source of the interrupt, reset the counter to 0.
        reg_write(ADDRESS, 0);
    }
    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}

