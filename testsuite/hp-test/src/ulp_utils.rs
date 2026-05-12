/// Test utils for ULP stuff
use embedded_hal::delay::DelayNs;
pub use esp_hal::ulp_core::{
    UlpCore as LpCore,
    UlpCoreTimerCycles as LpCoreTimerCycles,
    UlpCoreWakeupSource as LpCoreWakeupSource,
};
use esp_hal::{delay::Delay, load_lp_code};
use shared::{UlpCommand, UlpCommandType, UlpLoopCounter, UlpReply};

// Type aliasing for peripheral type
pub type LpCorePeripheral = esp_hal::peripherals::ULP_RISCV_CORE<'static>;

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

#[doc(hidden)]
fn ulp_timer_period(cycles: u32) {
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

pub fn reprogram_ulp_core(
    ulp_core: &mut LpCore,
    wakeup_source: LpCoreWakeupSource,
    command: UlpCommandType,
) {
    ulp_riscv_reset(); // this is required, to stop the ULP core from doing stuff while we program it.
    let ulp_code = load_lp_code!("lp_app");
    UlpLoopCounter::reset();
    UlpCommand::write(command);
    UlpReply::write(shared::UlpReplyType::UNKNOWN);
    ulp_code.run(ulp_core, wakeup_source);
}

pub fn ulp_is_running() -> bool {
    let a = UlpLoopCounter::read();
    Delay::new().delay_ms(500);
    let b = UlpLoopCounter::read();
    defmt::info!("a =  {}, b = {}", a, b);
    a != b
}
