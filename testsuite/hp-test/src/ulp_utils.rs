/// Test utils for ULP stuff
use embedded_hal::delay::DelayNs;
pub use esp_hal::ulp_core::{
    UlpCore as LpCore,
    UlpCoreTimerCycles as LpCoreTimerCycles,
    UlpCoreWakeupSource as LpCoreWakeupSource,
};
use esp_hal::{delay::Delay, load_lp_code, time::Instant};
use shared::{
    SharedType,
    UlpBootCounter,
    UlpCommand,
    UlpLock,
    UlpLoopCounter,
    UlpReply,
    ULP_TEST_DATA_IN,
    ULP_TEST_DATA_OUT,
};

// Type aliasing for peripheral type
pub type LpCorePeripheral = esp_hal::peripherals::ULP_RISCV_CORE<'static>;

// Minimum delay time to wait for processor to boot (tuned manually)
const ULP_HAS_BOOTED_DELAY_MILLIS: u32 = 1;
// Longest amount of time to wait for the loop counter to increment.
const ULP_IS_LOOPING_TIMEOUT_MILLIS: u64 = 1000;

pub fn ulp_riscv_timer_stop() {
    let rtc_cntl = esp_hal::peripherals::LPWR::regs();
    rtc_cntl
        .ulp_cp_timer()
        .write(|w| w.ulp_cp_slp_timer_en().clear_bit());
}

pub fn ulp_riscv_timer_resume() {
    let rtc_cntl = esp_hal::peripherals::LPWR::regs();
    rtc_cntl
        .ulp_cp_timer()
        .write(|w| w.ulp_cp_slp_timer_en().set_bit());
}

pub fn ulp_timer_period(cycles: u32) {
    let rtc_cntl = esp_hal::peripherals::LPWR::regs();
    rtc_cntl
        .ulp_cp_timer_1()
        .write(|w| unsafe { w.ulp_cp_timer_slp_cycle().bits(cycles << 8) });
    rtc_cntl
        .ulp_cp_ctrl()
        .modify(|_, w| w.ulp_cp_force_start_top().clear_bit());
}

pub fn ulp_riscv_halt() {
    ulp_riscv_timer_stop();
    let rtc_cntl = esp_hal::peripherals::LPWR::regs();
    // suspends the ulp operation
    rtc_cntl
        .cocpu_ctrl()
        .modify(|_, w| w.cocpu_done().set_bit());
    // Resets the processor
    rtc_cntl
        .cocpu_ctrl()
        .modify(|_, w| w.cocpu_shut_reset_en().set_bit());
}

pub fn ulp_riscv_reset() {
    let rtc_cntl = esp_hal::peripherals::LPWR::regs();

    rtc_cntl.cocpu_ctrl().write(|w| {
        w.cocpu_shut().clear_bit();
        w.cocpu_done().clear_bit();
        w.cocpu_shut_reset_en().clear_bit()
    });

    Delay::new().delay_us(20);

    rtc_cntl.cocpu_ctrl().write(|w| {
        w.cocpu_shut().set_bit();
        w.cocpu_done().set_bit();
        w.cocpu_shut_reset_en().set_bit()
    });

    Delay::new().delay_us(20);
}

/// UNDOCUMENTED ULP HARD-RESET PROCEDURE
/// Will recue the ULP core no matter how stuck it is,
/// and erase any previous firmware.
pub fn ulp_riscv_hard_reset() {
    let sar_ctrl = esp_hal::peripherals::SENS::regs();

    // Hard reset the coprocessor
    sar_ctrl
        .sar_peri_reset_conf()
        .write(|w| w.sar_cocpu_reset().set_bit());

    sar_ctrl
        .sar_peri_reset_conf()
        .write(|w| w.sar_cocpu_reset().clear_bit());

    // Erase ULP core region
    let lp_ram = unsafe { core::slice::from_raw_parts_mut(0x5000_0000 as *mut u32, 8 * 1024 / 4) };
    lp_ram.fill(0u32);
}

// Reset all of the shared variables to their uninitialised state.
// This can be called within the pre_run_hook.
fn reset_ulp_shared_variables() {
    UlpBootCounter::reset();
    UlpLoopCounter::reset();
    UlpReply::UNSET.store();
    UlpCommand::UNSET.store();
    unsafe {
        ULP_TEST_DATA_IN = 0;
        ULP_TEST_DATA_OUT = 0;
    }
    UlpLock::reset();
}

pub fn reprogram_ulp_core_with_run_hook<F>(
    ulp_core: &mut LpCore,
    wakeup_source: LpCoreWakeupSource,
    pre_run_hook: F,
) where
    F: FnOnce(),
{
    // this is required, to stop the ULP core from doing stuff while we program it.
    ulp_riscv_reset();

    let ulp_code = load_lp_code!("lp_app");
    // All shared variables are reset before reprogramming.
    reset_ulp_shared_variables();
    pre_run_hook();
    ulp_code.run(ulp_core, wakeup_source);
}

#[allow(static_mut_refs)]
pub fn ulp_has_booted() -> bool {
    Delay::new().delay_ms(ULP_HAS_BOOTED_DELAY_MILLIS);
    UlpBootCounter::load() != 0
}

#[allow(static_mut_refs)]
pub fn ulp_is_looping() -> bool {
    let t0 = Instant::now();
    let mut t1;
    let a = UlpLoopCounter::load();
    let mut b;

    loop {
        Delay::new().delay_us(10);

        t1 = Instant::now();
        b = UlpLoopCounter::load();

        if a != b {
            break;
        }
        if (t1 - t0).as_millis() >= ULP_IS_LOOPING_TIMEOUT_MILLIS {
            break;
        }
    }

    // Calculate rate difference
    let dt = (t1 - t0).as_micros();
    let c = (b - a) as u64;

    if c == 0 {
        defmt::println!("\na =  {}, b = {}. Timed out.", a, b);
        defmt::println!("\na =  {}, b = {}, rate = 0 Hz", a, b);
    } else {
        let count_rate = (c * 1000000) / dt;
        defmt::println!("\na =  {}, b = {}, rate = {} Hz", a, b, count_rate);
    }

    a != b
}
