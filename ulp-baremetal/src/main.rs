//! Increments a 32 bit counter value at a known point in memory, once a second.
//! Uses riscv_rt to provide interrupt handling, although it's linker scripts are overridden
//! with ones inside of ./ld/.

#![no_std]
#![no_main]

extern crate panic_halt;
use esp_lp_hal::prelude::*;
use esp_lp_hal::core_interrupt;
use esp_lp_hal::Interrupt;
use esp_lp_hal::delay::Delay;

// TODO: Figure out how to re-write _dispatch_core_interrupt and friends,
//       within esp-lp-hal, so that these interrupts work!
// Examples for how to use the interrupt attributes
// #[riscv_rt::exception(riscv::interrupt::Exception::LoadMisaligned)]
// fn load_misaligned(trap_frame: &mut riscv_rt::TrapFrame) -> ! {
//     loop{};
// }
// #[riscv_rt::core_interrupt(riscv::interrupt::Interrupt::SupervisorSoft)]
// fn supervisor_soft() -> ! {
//     loop{};
// }
// 
// #[riscv_rt::external_interrupt(e310x::interrupt::Interrupt::GPIO0)]
// fn gpio0() -> ! {
//     loop{};
// }

const ADDRESS: u32 = 0x1000;

#[entry]
fn main() -> ! {
    let dly = Delay{};
    let ptr = ADDRESS as *mut u32;
    let mut i : u32 = unsafe { ptr.read_volatile() };

    loop {
        i = i.wrapping_add(1u32);
        unsafe { ptr.write_volatile(i); }
        dly.delay_millis(100);
    }
}
