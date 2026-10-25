#![no_std]
#![no_main]
#![allow(static_mut_refs)]

#[allow(unused_imports)]
use esp_lp_hal::interrupt::{ExternalInterrupt, external_interrupt};
use esp_lp_hal::{delay::Delay, prelude::*};
use panic_halt as _;
use smart_leds::{
    SmartLedsWrite,
    colors::BLACK,
    hsv::{Hsv, hsv2rgb},
};
use ws2812_esp32s3_ulp::Ws2812;
mod colours;
use colours::apply_brightness;

#[unsafe(no_mangle)]
#[used]
pub static mut RAINBOW_COUNTER: u32 = 0;

// Rainbow run will be toggled when the HP core is awake.
// true == cycle rainbow
#[unsafe(no_mangle)]
#[used]
pub static mut RAINBOW_RUN: bool = false;

#[allow(unused)]
fn enable_button_interrupts() {
    let btn_reg = unsafe { &*esp_lp_hal::pac::RTC_IO::PTR };
    btn_reg.touch_pad5().write(|w| unsafe {
        w.mux_sel()
            .set_bit()
            .fun_ie()
            .set_bit()
            .rue()
            .clear_bit()
            .rde()
            .clear_bit()
            .fun_sel()
            .bits(0)
    });
    // 1 = rising
    // 2 = falling
    // 3 = any
    btn_reg.pin5().write(|w| unsafe { w.int_type().bits(1) });
}

#[entry]
fn main(gpio18_led: esp_lp_hal::gpio::Output<18>) {
    let count = unsafe { RAINBOW_COUNTER };
    let is_paused = !unsafe { RAINBOW_RUN };

    // enable_button_interrupts();

    let ws_clk = Delay {};
    let mut ws = Ws2812::new(gpio18_led, ws_clk);

    if is_paused {
        // SPECIAL CASE
        // If is_paused on the first count (0), then we will turn the LED off.
        // This is how the HP core 'disables' the RGB led, for ultra-low-power operation.
        if count == 0 {
            let _ = critical_section::with(|_cs| ws.write([BLACK]));
            // Increment to prevent needing to write the same data over and over
            unsafe {
                RAINBOW_COUNTER = count + 1;
            }
        }
    } else {
        let hsv = Hsv {
            hue: (count & 0xFF) as u8,
            sat: 255,
            val: 64,
        };
        let rgb = apply_brightness(hsv2rgb(hsv), 16);
        // Prevent the GPIO interrupt from getting in the way of our LED write!
        // Without this, you can spam the GPIO button, and the LED will glitch
        let _ = critical_section::with(|_cs| ws.write([rgb]));
        // increment counter when not paused
        unsafe {
            RAINBOW_COUNTER = count + 1;
        }
    }
}

// #[inline(always)]
// fn cycles() -> u64 {
//     let mut cycles: u32;
//     unsafe {
//         core::arch::asm!(
//             "rdcycle {cycles}",
//             cycles = out(reg) cycles,
//         )
//     }
//     cycles as u64
// }

// #[external_interrupt(ExternalInterrupt::GpioInterrupt)]
// unsafe fn gpio_int() {
//     const VOTE_THRESH: u32 = 50;
//     let mut press_votes = 0;
//     // read the inputs up to 100 times.
//     // if we read '1', increment press_votes by 1.
//     // if we read '0', reset press_votes to 0.
//     // if press_votes >= 5, break.
//     for _ in 0..100 {
//         let inputs = unsafe { &*esp_lp_hal::pac::RTC_IO::PTR }
//             .in_()
//             .read()
//             .bits()
//             >> 10;
//         // gpio 5, on press interrupt
//         if (inputs & (1 << 5)) != 0 {
//             press_votes += 1;
//         }
//         if press_votes >= VOTE_THRESH {
//             break;
//         }
//         // debounce by 100 us
//         const DELAY: u64 = 17_500_000 / 10000;
//         let t0 = cycles();
//         while cycles().wrapping_sub(t0) <= DELAY {}
//     }
//     if press_votes >= VOTE_THRESH {
//         esp_lp_hal::wake_hp_core();
//     }
// }
