#![no_std]
#![no_main]
#![allow(static_mut_refs)]

// use embedded_hal::i2c::I2c;
use esp_lp_hal::{
    delay::Delay,
    // gpio,
    interrupt::{
        CoreInterrupt,
        Exception,
        ExternalInterrupt,
        TrapFrame,
        core_interrupt,
        exception,
        external_interrupt,
        // ulp_maskirq,
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
// use riscv_rt::InterruptNumber;
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
use shared::{TEST_XOR_MASK, UlpCmd, UlpReply};

// GLOBAL SHARED VARIABLES
#[shared]
pub static ULP_HALT_COUNTER: shared::UlpCounter = shared::UlpCounter::new();
#[shared]
pub static ULP_LOOP_COUNTER: shared::UlpCounter = shared::UlpCounter::new();
#[shared]
pub static ULP_BOOT_COUNTER: shared::UlpCounter = shared::UlpCounter::new();
#[shared]
pub static ULP_COMMAND: shared::UlpCmdWrapper = shared::UlpCmdWrapper::new();
#[shared]
pub static ULP_REPLY: shared::UlpReplyWrapper = shared::UlpReplyWrapper::new();
#[shared]
pub static ULP_LOCK: shared::UlpLockWrapper = shared::UlpLockWrapper::new();
#[shared]
pub static ULP_DEBUG_GPIO_ISR_COUNT: shared::UlpCounter = shared::UlpCounter::new();
#[shared]
pub static ULP_DEBUG_GPIO_ISR_STATUS: shared::UlpCounter = shared::UlpCounter::new();
#[shared]
pub static ULP_DEBUG_LAST_ISR_DATA: shared::UlpCounter = shared::UlpCounter::new();
#[shared]
pub static ULP_TEST_DATA_IN: shared::UlpCounter = shared::UlpCounter::new();
#[shared]
pub static ULP_TEST_DATA_OUT: shared::UlpCounter = shared::UlpCounter::new();

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

// Return rdcycle value, 32 bit only.
// Will wrap after about ~245 seconds.
#[inline(always)]
fn cycles32() -> u32 {
    let mut cycles: u32;
    unsafe {
        core::arch::asm!(
            "rdcycle {cycles}",
            cycles = out(reg) cycles,
        )
    }
    cycles
}

fn delay_for_a_tenth_second() {
    const DELAYYY: u64 = 17_500_000 / 10;
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
    reg.touch_pad0()
        .write(|w| w.fun_ie().set_bit().mux_sel().set_bit());
    reg.touch_pad1()
        .write(|w| w.fun_ie().set_bit().mux_sel().set_bit());
    reg.pin0().write(|w| w.pad_driver().set_bit());
    reg.pin1().write(|w| w.pad_driver().set_bit());
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
    ULP_BOOT_COUNTER.increment();

    loop {
        // Re-reading the command allows it to be modified by ourself.
        // E.g. MUTEX_TEST will change the command to NOOP when completed,
        // to prevent re-running the test.
        let cmd = ULP_COMMAND.load();

        match cmd {
            UlpCmd::NOOP => {
                // Do nothing
                ULP_REPLY.store(UlpReply::OK);
            }
            UlpCmd::COUNTER_ONESHOT => {
                // Increment counter ONCE, then stay loop-ing on NOOP.
                ULP_LOOP_COUNTER.increment();
                ULP_COMMAND.store(UlpCmd::NOOP);
                ULP_REPLY.store(UlpReply::OK);
            }
            UlpCmd::COUNTER_LOOP => {
                // Keep incrementing the counter in a loop
                ULP_LOOP_COUNTER.increment();
                ULP_REPLY.store(UlpReply::OK);
            }
            UlpCmd::COUNTER_ULP_TIMER => {
                ULP_LOOP_COUNTER.increment();
                ULP_REPLY.store(UlpReply::OK);
                // Exit the loop, the ULP Timer will re-start us.
                break;
            }
            UlpCmd::XOR_TEST => {
                ULP_LOOP_COUNTER.increment();
                let data_in = ULP_TEST_DATA_IN.load();
                ULP_TEST_DATA_OUT.store(data_in ^ TEST_XOR_MASK);
                // Run once.
                ULP_COMMAND.store(UlpCmd::NOOP);
                ULP_REPLY.store(UlpReply::OK);
            }
            UlpCmd::STOP_TEST => {
                ULP_LOOP_COUNTER.increment();
                ULP_REPLY.store(UlpReply::OK);
                ulp_riscv_timer_stop();
                break;
            }
            UlpCmd::MUTEX_TEST => {
                // Reset status to indicate we are processing
                ULP_REPLY.store(UlpReply::BUSY);
                let iters = ULP_TEST_DATA_IN.load();
                for _ in 0..iters {
                    ULP_LOCK.acquire();
                    ULP_LOOP_COUNTER.increment();
                    ULP_LOCK.release();
                }
                ULP_REPLY.store(UlpReply::OK);
                ULP_COMMAND.store(UlpCmd::NOOP);
            }
            UlpCmd::TIMER_PERIOD_TEST => {
                ULP_LOOP_COUNTER.increment();
                let new_cycles = ULP_TEST_DATA_IN.load();
                ulp_timer_period(new_cycles);
                ULP_REPLY.store(UlpReply::OK);
                break;
            }
            UlpCmd::LIGHT_SLEEP_TEST => {
                ULP_LOOP_COUNTER.increment();
                ULP_REPLY.store(UlpReply::OK);

                // NEW approach - wait for HP core to release the lock.
                // HP core has 0.1 seconds to acquire before we do.
                delay_for_a_tenth_second();
                // BLOCK HERE UNTIL HP CORE RELEASES
                ULP_LOCK.acquire();
                // Wake up the core
                wake_hp_core();
                // Disable the timer, so the LP-core will remain halted on exit.
                ulp_riscv_timer_stop();
                break;
            }
            UlpCmd::EXCEPTION_TEST => {
                // Should trigger an illegal instruction
                ULP_LOOP_COUNTER.increment();
                ULP_REPLY.store(UlpReply::OK);
                unsafe {
                    core::arch::asm!("csrrs a1, mcause, zero");
                }
            }
            UlpCmd::START_INT_TEST => {
                if ULP_LOOP_COUNTER.load() == 0 {
                    ULP_REPLY.store(UlpReply::OK);
                    // Enable the COCPU start interrupt
                    let reg = unsafe { &*esp_lp_hal::pac::SENS::PTR };
                    reg.sar_cocpu_int_ena()
                        .write(|w| w.sar_cocpu_start_int_ena().set_bit());
                }
                // Increment the loop counter
                ULP_LOOP_COUNTER.increment();
                // Add a delay to make the counter chill out
                delay_for_a_tenth_second();
            }
            UlpCmd::WAITIRQ_TIMER_TEST => {
                // ULP_TEST_DATA_OUT --> approx number of CPU cycles
                // that were elapsed while waitirq blocked.
                // ULP_TEST_DATA_IN <-- TIMER_PER duration to run the timer
                // for TIMER_PER is in clock cycles, which is 17.5MHz.
                let timer_period: u32 = ULP_TEST_DATA_IN.load();
                // Clear the previous test data state
                ULP_TEST_DATA_OUT.store(0);
                // This will run BUSY -> OK -> BUSY -> OK
                ULP_REPLY.store(UlpReply::BUSY);
                // Start the timer
                ulp_timer_insn(timer_period);
                // Get our cpu cycle count before blocking on waitirq
                let t0 = cycles32();
                // Call 'waitirq' to block until the timer interrupt fires
                let irqm = ulp_wait_irq();
                // Get cpu cycles after blocking
                let t1 = cycles32();
                // Store the number of cycles we blocked for
                ULP_TEST_DATA_OUT.store(t1.wrapping_sub(t0));
                // If the irq flag is 1, it means the timer expired.
                if (irqm & 1) != 0 {
                    // Disable the timer (unsure if needed?)
                    ulp_timer_insn(0);
                    ULP_REPLY.store(UlpReply::OK);
                    ULP_LOOP_COUNTER.increment();
                } else {
                    // Waitirq did catch the timer IRQ, test has failed
                    ULP_REPLY.store(UlpReply::NOK);
                }
                // Change to NO-OP
                ULP_COMMAND.store(UlpCmd::NOOP);
            }
            UlpCmd::GPIO_INT_TEST => {
                if ULP_LOOP_COUNTER.load() == 0 {
                    ULP_REPLY.store(UlpReply::OK);
                }
                // Increment the loop counter
                ULP_LOOP_COUNTER.increment();
                // Add a delay to make the counter chill out
                delay_for_a_tenth_second();
            }
            UlpCmd::GPIO_WAKEUP_TEST => unsafe {
                ULP_LOOP_COUNTER.increment();
                ULP_REPLY.store(UlpReply::OK);
                // Wake the HP core too (not used in the test suite, but useful in other examples)
                esp_lp_hal::wake_hp_core();
                // ULP will keep booting while the GPIO wake-up event is true,
                // and the gpio wakeup event hasn't been cleared.
                setup_gpio_wakeup(true);
                break;
            },
            UlpCmd::MAX1708_I2C_BMS_TEST => unsafe {
                ULP_REPLY.store(UlpReply::BUSY);
                let (scl, sda) = steal_i2c_pins();
                let i2c_clk = Timer::new();
                let i2c = I2cBB::new(scl, sda, i2c_clk);
                let mut max1708 = Max17048::new(i2c);
                if let Ok(v) = max1708.voltage() {
                    {
                        ULP_TEST_DATA_OUT.store(v as u32);
                    }
                }
                ULP_LOOP_COUNTER.increment();
                Delay {}.delay_millis(10);
                ULP_COMMAND.store(UlpCmd::NOOP);
            },
            UlpCmd::WS2812_LED_TEST => unsafe {
                ULP_REPLY.store(UlpReply::OK);
                let gpio18_led = steal_output::<18>();
                let ws_clk = Timer::new();
                let mut ws = Ws2812::new(gpio18_led, ws_clk);
                let hsv = Hsv {
                    hue: ULP_BOOT_COUNTER.load() as u8,
                    sat: 240,
                    val: 240,
                };
                let rgb = apply_brightness(hsv2rgb(hsv), 10);
                let _ = ws.write([rgb]);
                ULP_LOOP_COUNTER.increment();
                // Break and go to sleep
                break;
            },
            _ => {
                // Unknown command, not okay!
                ULP_REPLY.store(UlpReply::NOK);
            }
        };
    }

    ULP_HALT_COUNTER.increment();
}

// Used for EXCEPTION_TEST
#[exception(Exception::IllegalInstruction)]
unsafe fn illegal_instruction(_trap: &TrapFrame) -> ! {
    ULP_DEBUG_LAST_ISR_DATA.store(0xdeadbeef);
    #[allow(clippy::empty_loop)]
    loop {}
}

#[exception(Exception::LoadMisaligned)]
unsafe fn misaligned_load(_trap: &TrapFrame) -> ! {
    ULP_DEBUG_LAST_ISR_DATA.store(0x0000beef);
    #[allow(clippy::empty_loop)]
    loop {}
}

#[core_interrupt(CoreInterrupt::MachineTimer)]
unsafe fn machine_timer() {
    ULP_LOOP_COUNTER.increment();
    ULP_DEBUG_LAST_ISR_DATA.store(0xba5eba11);
}

// Used for START_INT_TEST
#[external_interrupt(ExternalInterrupt::SensInterrupt)]
unsafe fn sens_interrupt() {
    let sens_int = unsafe { &*esp_lp_hal::pac::SENS::PTR }
        .sar_cocpu_int_st()
        .read();
    if sens_int.sar_cocpu_start_int_st().bit_is_set() {
        ULP_DEBUG_LAST_ISR_DATA.store(0xcafebabe);
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
    ULP_DEBUG_GPIO_ISR_COUNT.increment();
    ULP_DEBUG_GPIO_ISR_STATUS.store(rtcio_int_st.bits()); // Trap data holds the ISR status bits
    ULP_DEBUG_LAST_ISR_DATA.store(rtcio_level_status.bits()); // ISR data holds the pin level bits
}
