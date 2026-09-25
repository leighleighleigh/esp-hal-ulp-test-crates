//! Increments a 32 bit counter value at a known point in memory, once a second.
#![no_std]
#![no_main]
#![allow(unused)]
#![allow(static_mut_refs)]

use core::iter;

use esp_lp_hal::{
    delay::Delay,
    interrupt::{
        core_interrupt,
        exception,
        external_interrupt,
        CoreInterrupt,
        Exception,
        ExternalInterrupt,
        TrapFrame,
    },
    prelude::*,
    ulp_riscv_timer_stop,
    ulp_timer_period,
    wake_hp_core,
};
use panic_halt as _;
use riscv_rt::InterruptNumber;
use shared::{
    SharedType,
    UlpBootCounter,
    UlpCommand,
    UlpHaltCounter,
    UlpLock,
    UlpLoopCounter,
    UlpReply,
    TEST_MUTEX_ITERATIONS,
    TEST_XOR_MASK,
    ULP_DEBUG_ISR_DATA,
    ULP_DEBUG_TRAP_DATA,
    ULP_TEST_DATA_IN,
    ULP_TEST_DATA_OUT,
};

#[inline(always)]
fn cycles() -> u64 {
    let mut cycles: u32;
    unsafe {
        core::arch::asm!(
            "rdcycle {cycles}",
            cycles = out(reg) cycles,
        )
    }

    cycles as u64
}

#[inline]
fn delay_using_loop(iterations: u32) {
    for i in 0..iterations {
        unsafe {
            core::arch::asm!("nop");
        }
    }
}

fn delay_for_a_hundred_microseconds() {
    const DELAYYY: u64 = 1750;
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= DELAYYY {}
}

fn delay_for_a_tenth_second() {
    const DELAYYY: u64 = 17_500_000 / 10;
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= DELAYYY {}
}

fn delay_for_a_second() {
    const DELAYYY: u64 = 17_500_000;
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= DELAYYY {}
}

#[entry]
fn main() {
    UlpBootCounter::increment();

    loop {
        // Re-reading the command allows it to be modified by ourself.
        // E.g. MUTEX_TEST will change the command to NOOP when completed,
        // to prevent re-running the test.
        let cmd: UlpCommand = UlpCommand::load();

        match cmd {
            UlpCommand::NOOP => {
                // Do nothing
                UlpReply::OK.store();
            }
            UlpCommand::COUNTER_ONESHOT => {
                // Increment counter ONCE, then stay loop-ing on NOOP.
                UlpLoopCounter::increment();
                UlpCommand::NOOP.store();
                UlpReply::OK.store();
            }
            UlpCommand::COUNTER_LOOP => unsafe {
                // Keep incrementing the counter in a loop
                UlpLoopCounter::increment();
                UlpReply::OK.store();
            },
            UlpCommand::COUNTER_ULP_TIMER => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                // Exit the loop, the ULP Timer will re-start us.
                break;
            },
            UlpCommand::XOR_TEST => unsafe {
                UlpLoopCounter::increment();
                let data_in = unsafe { ULP_TEST_DATA_IN };
                unsafe { ULP_TEST_DATA_OUT = data_in ^ TEST_XOR_MASK };
                // Run once.
                UlpCommand::NOOP.store();
                UlpReply::OK.store();
            },
            UlpCommand::STOP_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                ulp_riscv_timer_stop();
                break;
            },
            UlpCommand::MUTEX_TEST => unsafe {
                for _ in 0..TEST_MUTEX_ITERATIONS {
                    UlpLock::acquire();
                    UlpLoopCounter::increment();
                    UlpLock::release();
                }
                UlpCommand::NOOP.store();
                UlpReply::OK.store();
            },
            UlpCommand::TIMER_PERIOD_TEST => unsafe {
                UlpLoopCounter::increment();
                let new_cycles = unsafe { ULP_TEST_DATA_IN };
                ulp_timer_period(new_cycles);
                UlpReply::OK.store();
                break;
            },
            UlpCommand::LIGHT_SLEEP_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();

                // NEW approach - wait for HP core to release the lock.
                // HP core has 0.1 seconds to acquire before we do.
                delay_for_a_tenth_second();
                // BLOCK HERE UNTIL HP CORE RELEASES
                UlpLock::acquire();
                // Wake up the core
                wake_hp_core();
                // Disable the timer, so the LP-core will remain halted on exit.
                ulp_riscv_timer_stop();
                break;
            },
            UlpCommand::EXCEPTION_TEST => unsafe {
                // Should trigger an illegal instruction
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                unsafe {
                    core::arch::asm!("csrrs a1, mcause, zero");
                }
            },
            UlpCommand::START_INT_TEST => unsafe {
                if UlpLoopCounter::load() == 0 {
                    UlpReply::OK.store();
                    // Enable the COCPU start interrupt
                    let reg = unsafe { &*esp_lp_hal::pac::SENS::PTR };
                    reg.sar_cocpu_int_ena()
                        .write(|w| w.sar_cocpu_start_int_ena().set_bit());
                }
                // Increment the loop counter
                UlpLoopCounter::increment();
                // Add a delay to make the counter chill out
                delay_for_a_tenth_second();
            },
            UlpCommand::GPIO_INT_TEST => unsafe {
                if UlpLoopCounter::load() == 0 {
                    UlpReply::OK.store();
                    // Technically either of the HP or LP cores may configure the RTC Pin,
                    // but I've opted to let the HP core do it, as it's more 'in charge'.

                    // // Enable GPIO interrupt for the pushbutton
                    // let reg = unsafe { &*esp_lp_hal::pac::RTC_IO::PTR };
                    // // Configure RTC Pin5.
                    // reg.touch_pad5().write(|w| {
                    //     w.mux_sel()
                    //         .set_bit()
                    //         .fun_ie()
                    //         .set_bit()
                    //         .rue()
                    //         .clear_bit()
                    //         .rde()
                    //         .clear_bit()
                    //         .fun_sel()
                    //         .bits(0)
                    // });
                    // // Enable rising edge interrupt on pin5
                    // reg.pin5().write(|w| w.int_type().bits(1));
                }
                // Increment the loop counter
                UlpLoopCounter::increment();
                // Add a delay to make the counter chill out
                delay_for_a_tenth_second();
            },
            _ => unsafe {
                // Unknown command, not okay!
                UlpReply::NOK.store();
            },
        };
    }

    UlpHaltCounter::increment();
}

// Used for EXCEPTION_TEST
#[exception(Exception::IllegalInstruction)]
unsafe fn illegal_instruction(_trap: &TrapFrame) -> ! {
    unsafe { ULP_DEBUG_ISR_DATA = 0xdeadbeef };
    #[allow(clippy::empty_loop)]
    loop {}
}

#[exception(Exception::LoadMisaligned)]
unsafe fn misaligned_load(_trap: &TrapFrame) -> ! {
    unsafe { ULP_DEBUG_ISR_DATA = 0x0000beef };
    #[allow(clippy::empty_loop)]
    loop {}
}

// Used for START_INT_TEST
#[external_interrupt(ExternalInterrupt::SensInterrupt)]
unsafe fn sens_interrupt() {
    let sens_int = unsafe { &*esp_lp_hal::pac::SENS::PTR }
        .sar_cocpu_int_st()
        .read();
    if sens_int.sar_cocpu_start_int_st().bit_is_set() {
        unsafe { ULP_DEBUG_ISR_DATA = 0xcafebabe };
        // Disable the COCPU start interrupt,
        // to prevent it happening again.
        let reg = unsafe { &*esp_lp_hal::pac::SENS::PTR };
        reg.sar_cocpu_int_ena()
            .write(|w| w.sar_cocpu_start_int_ena().clear_bit());
    }
}

// Used for GPIO_INT_TEST
#[external_interrupt(ExternalInterrupt::GpioInterrupt)]
unsafe fn gpio_interrupt() {
    let rtcio_level_status = unsafe { &*esp_lp_hal::pac::RTC_IO::PTR }.in_().read();
    let rtcio_int_st = unsafe { &*esp_lp_hal::pac::RTC_IO::PTR }.status().read();
    // GPIO interrupts must be right shifted by 10,
    // so that Bit 0 == GPIO 0.
    unsafe {
        ULP_DEBUG_TRAP_DATA = rtcio_int_st.bits(); // Trap data holds the ISR status bits
        ULP_DEBUG_ISR_DATA = rtcio_level_status.bits(); // ISR data holds the pin level bits
    };
}
