/// Test utils for ULP stuff
use embedded_hal::delay::DelayNs;
use esp_hal::delay::Delay;

use shared::{RW, UlpLoopCounter};

use esp_hal::load_lp_code;
pub use esp_hal::ulp_core::{
    UlpCore as LpCore,
    UlpCoreTimerCycles as LpCoreTimerCycles,
    UlpCoreWakeupSource as LpCoreWakeupSource,
};

// Type aliasing for peripheral type
pub type LpCorePeripheral = esp_hal::peripherals::ULP_RISCV_CORE<'static>;

pub fn ulp_riscv_timer_stop()
{
    let rtc_cntl = esp_hal::peripherals::LPWR::regs();
    rtc_cntl.ulp_cp_timer().write(|w| w.ulp_cp_slp_timer_en().clear_bit());
}

pub fn ulp_riscv_timer_resume()
{
    let rtc_cntl = esp_hal::peripherals::LPWR::regs();
    rtc_cntl.ulp_cp_timer().write(|w| w.ulp_cp_slp_timer_en().set_bit());
}


pub fn ulp_riscv_halt()
{
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
}

pub fn start_ulp_core(core: LpCorePeripheral, wakeup_source : LpCoreWakeupSource) {
    ulp_riscv_timer_stop();
    ulp_riscv_halt();
    let mut ulp_core = LpCore::new(core);
    let ulp_core_code = load_lp_code!("lp_app");

    // Reset counter
    UlpLoopCounter::from_raw(0).save();

    // ulp_core_code.run(&mut ulp_core, LpCoreWakeupSource::Timer(LpCoreTimerCycles::new(53)));
    ulp_core_code.run(&mut ulp_core, wakeup_source);
}

pub fn ulp_is_running() -> bool {
    let a = UlpLoopCounter::load().into_raw();
    Delay::new().delay_ms(500);
    let b = UlpLoopCounter::load().into_raw();
    defmt::info!("a =  {}, b = {}", a, b);
    a != b
}
