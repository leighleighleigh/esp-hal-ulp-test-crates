//! Increments a 32 bit counter value at a known point in memory, once a second.
//! Operated with riscv_rt provided entrypoints

#![no_std]
#![no_main]

//use panic_halt as _;
extern crate panic_halt;
use riscv_rt::entry;
use core::arch::global_asm;

// Include macros which define custom RISCV R-Type instructions, used for interrupt handling.
global_asm!(include_str!("./ulp_riscv_interrupt_ops.S"));

// Assembly containing the reset_vector and irq_vector instructions.
// TODO: The irq_vector here should be hooked into the pre_start_trap stuff somehow, probably
global_asm!(include_str!("./ulp_riscv_vectors.S"));

// TODO: This should be hooked into pre_init / post_init somehow, so it can call the maskirq_insn 
//global_asm!(
//    r#"
//    .balign 0x10
//    .section .init
//    __start:
//        /* setup the stack pointer */
//        la sp, __stack_top
//
//        /* Custom instruction to un-mask the interrupts */
//        /* waitirq_insn zero */
//        maskirq_insn zero, zero
//
//        call rust_main
//    "#
//);

const ADDRESS: u32 = 0x1000;

#[entry]
fn main() -> ! {
    let ptr = ADDRESS as *mut u32;
    let mut i : u32 = unsafe { ptr.read_volatile() };
    i = i.wrapping_add(1u32);
    unsafe { ptr.write_volatile(i); }

    loop {}
}
