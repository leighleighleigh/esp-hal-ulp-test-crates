#![no_std]
#![no_main]
#![allow(unused)]
#![allow(static_mut_refs)]

use core::iter;

use embedded_hal::i2c::I2c;
use esp_lp_hal::{
    delay::Delay,
    gpio,
    interrupt::{
        CoreInterrupt,
        Exception,
        ExternalInterrupt,
        TrapFrame,
        core_interrupt,
        exception,
        external_interrupt,
        ulp_maskirq,
        ulp_timer_insn,
        ulp_wait_irq,
    },
    prelude::*,
    ulp_riscv_timer_stop,
    ulp_timer_period,
    wake_hp_core,
};
use max170xx::Max17048;
use panic_halt as _;
use riscv_rt::InterruptNumber;
use shared::{
    TEST_MUTEX_ITERATIONS,
    TEST_XOR_MASK,
    ULP_DEBUG_GPIO_ISR_COUNT,
    ULP_DEBUG_GPIO_ISR_STATUS,
    ULP_DEBUG_LAST_ISR_DATA,
    ULP_TEST_DATA_IN,
    ULP_TEST_DATA_OUT,
    UlpBootCounter,
    UlpCommand,
    UlpHaltCounter,
    UlpLock,
    UlpLoopCounter,
    UlpReply,
    traits::*,
};
use smart_leds::{
    SmartLedsWrite,
    hsv::{Hsv, hsv2rgb},
};
use ws2812_esp32s3_ulp::Ws2812;

// Colours for ws2812
mod colours;
use colours::apply_brightness;

// Instant and Duration for lp hal
mod time;
// better timer for bit-banged devices
mod timer;
use timer::Timer;

// bit-banged i2c driver
mod i2cbb;
use i2cbb::I2cBB;

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

// Enable or disable GPIO wakeup
unsafe fn setup_gpio_wakeup(enabled: bool) {
    let reg = unsafe { &*esp_lp_hal::pac::RTC_CNTL::PTR };
    // Clear outstanding wakeup events
    reg.rtc_ulp_cp_timer()
        .write(|w| w.ulp_cp_gpio_wakeup_clr().set_bit());

    // Re-enable if desired
    if enabled {
        reg.rtc_ulp_cp_timer()
            .write(|w| w.ulp_cp_gpio_wakeup_ena().set_bit());
    }
}

// Steal a GPIO pin from the HP core by enabling IO_MUX
unsafe fn steal_i2c_pins() -> (
    esp_lp_hal::gpio::OutputOpenDrain<0>,
    esp_lp_hal::gpio::OutputOpenDrain<1>,
) {
    let reg = unsafe { &*esp_lp_hal::pac::RTC_IO::PTR };
    unsafe {
        reg.touch_pad0()
            .write(|w| w.fun_ie().set_bit().mux_sel().set_bit());
        reg.touch_pad1()
            .write(|w| w.fun_ie().set_bit().mux_sel().set_bit());
        reg.pin0().write(|w| w.pad_driver().set_bit());
        reg.pin1().write(|w| w.pad_driver().set_bit());
    }
    (
        esp_lp_hal::gpio::OutputOpenDrain::<0>::new(),
        esp_lp_hal::gpio::OutputOpenDrain::<1>::new(),
    )
}

unsafe fn steal_output<const PIN: u8>() -> esp_lp_hal::gpio::Output<PIN> {
    esp_lp_hal::gpio::Output::<PIN>::new()
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
                // Reset status to indicate we are processing
                UlpReply::BUSY.store();
                let iters = unsafe { ULP_TEST_DATA_IN };
                for _ in 0..iters {
                    UlpLock::acquire();
                    UlpLoopCounter::increment();
                    UlpLock::release();
                }
                UlpReply::OK.store();
                UlpCommand::NOOP.store();
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
            UlpCommand::WAITIRQ_TIMER_TEST => unsafe {
                // Start timer to be 200ms
                // timer is in clock cycles which are 17.5MHz.
                // delay for tenth of section is the /10
                const TIMER_PER: u32 = (17_500_000 / 10) * 2;
                // Clear the previous test data state
                unsafe {
                    ULP_TEST_DATA_IN = 0;
                }
                // This will run BUSY -> OK -> BUSY -> OK
                UlpReply::BUSY.store();

                // Start the timer
                ulp_timer_insn(TIMER_PER);

                // Use wait_irq to block until interrupt fires.
                let irqm = ulp_wait_irq();
                unsafe {
                    ULP_TEST_DATA_IN = irqm;
                }
                if (irqm & 1) != 0 {
                    ulp_timer_insn(0);
                    UlpLoopCounter::increment();
                }
                // Change to NO-OP
                UlpCommand::NOOP.store();
            },
            UlpCommand::GPIO_INT_TEST => unsafe {
                if UlpLoopCounter::load() == 0 {
                    UlpReply::OK.store();
                }
                // Increment the loop counter
                UlpLoopCounter::increment();
                // Add a delay to make the counter chill out
                delay_for_a_tenth_second();
            },
            UlpCommand::GPIO_WAKEUP_TEST => unsafe {
                UlpLoopCounter::increment();
                UlpReply::OK.store();
                // Wake the HP core too (not used in the test suite, but useful in other examples)
                esp_lp_hal::wake_hp_core();
                // ULP will keep booting while the GPIO wake-up event is true,
                // and the gpio wakeup event hasn't been cleared.
                setup_gpio_wakeup(true);
                break;
            },
            UlpCommand::MAX1708_I2C_BMS_TEST => unsafe {
                UlpReply::BUSY.store();
                let (mut scl, mut sda) = steal_i2c_pins();
                let i2c_clk = Timer::new();
                let mut i2c = I2cBB::new(scl, sda, i2c_clk);
                let mut max1708 = Max17048::new(i2c);
                match max1708.voltage() {
                    Ok(v) => unsafe {
                        ULP_TEST_DATA_OUT = v as u32;
                    },
                    Err(_) => {}
                }

                match max1708.reset() {
                    Ok(_) => unsafe {},
                    Err(_) => {}
                }

                UlpLoopCounter::increment();
                Delay {}.delay_millis(10);
                UlpCommand::NOOP.store();
            },
            UlpCommand::WS2812_LED_TEST => unsafe {
                UlpReply::OK.store();
                let mut gpio18_led = steal_output::<18>();
                let ws_clk = Timer::new();
                let mut ws = Ws2812::new(gpio18_led, ws_clk);
                let hsv = Hsv {
                    hue: UlpBootCounter::load() as u8,
                    sat: 240,
                    val: 240,
                };
                let rgb = apply_brightness(hsv2rgb(hsv), 10);
                ws.write([rgb]);
                UlpLoopCounter::increment();
                // Break and go to sleep
                break;
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
    unsafe { ULP_DEBUG_LAST_ISR_DATA = 0xdeadbeef };
    #[allow(clippy::empty_loop)]
    loop {}
}

#[exception(Exception::LoadMisaligned)]
unsafe fn misaligned_load(_trap: &TrapFrame) -> ! {
    unsafe { ULP_DEBUG_LAST_ISR_DATA = 0x0000beef };
    #[allow(clippy::empty_loop)]
    loop {}
}

#[core_interrupt(CoreInterrupt::MachineTimer)]
unsafe fn machine_timer() {
    UlpLoopCounter::increment();
    unsafe { ULP_DEBUG_LAST_ISR_DATA = 0xba5eba11 };
}

// Used for START_INT_TEST
#[external_interrupt(ExternalInterrupt::SensInterrupt)]
unsafe fn sens_interrupt() {
    let sens_int = unsafe { &*esp_lp_hal::pac::SENS::PTR }
        .sar_cocpu_int_st()
        .read();
    if sens_int.sar_cocpu_start_int_st().bit_is_set() {
        unsafe { ULP_DEBUG_LAST_ISR_DATA = 0xcafebabe };
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
        ULP_DEBUG_GPIO_ISR_COUNT += 1;
        ULP_DEBUG_GPIO_ISR_STATUS = rtcio_int_st.bits(); // Trap data holds the ISR status bits
        ULP_DEBUG_LAST_ISR_DATA = rtcio_level_status.bits(); // ISR data holds the pin level bits
    };
}
