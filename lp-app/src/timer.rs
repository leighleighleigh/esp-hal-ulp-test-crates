use embedded_hal::delay::DelayNs;

use crate::time::{Duration, Instant, Rate};

// For esp32s3 ULP RISCV
pub const CPU_CLOCK: u32 = 17_500_000;

// this is important, not doing this may cause stackoverflow
#[inline(always)]
pub fn cycles() -> u32 {
    let mut cycles: u32;
    unsafe {
        core::arch::asm!(
            "rdcycle {cycles}",
            cycles = out(reg) cycles,
        )
    }
    cycles
}

#[inline(always)]
pub fn cycles_hi() -> u32 {
    let mut cycles: u32;
    unsafe {
        core::arch::asm!(
            "rdcycleh {cycles}",
            cycles = out(reg) cycles,
        )
    }
    cycles
}

/// WARN: This is limited to u32 precision and will not work for large delay values.
#[inline(always)] // this is important, not doing this may cause stackoverflow
pub fn delay_cycles_u32(n: u32) {
    let t0 = cycles();
    while cycles().wrapping_sub(t0) <= n {}
}

#[inline(always)] // this is important, not doing this may cause stackoverflow
pub fn delay_cycles(n: u64) {
    if n < u32::MAX as u64 {
        delay_cycles_u32(n as u32);
    } else {
        let mut remain = n;
        while remain > 0 {
            let take = remain.min(u32::MAX as u64);
            delay_cycles_u32(take as u32);
            remain -= take;
        }
    }
}

// Time-based delay.
// This is an alternative to cycle_delay, which leverages the const-time cycles-to-seconds
// conversion provided by fugit. This should in theory be much more accurate than my 'divide by 17'
// approach taken in cycle_delay.
//
// The api of Timer is similar to that of the embassy_time::Timer,
// although with the added convenience of a DelayNs implimentation.

// A blocking delay utility.
pub struct Timer {}

impl Timer {
    pub fn new() -> Self {
        Timer {}
    }

    // Block execution for $duration amount of time.
    fn after(duration: Duration) {
        delay_cycles(duration.ticks());
    }

    // Block execution until a specific timestamp
    fn until(instant: Instant) {
        let n = Instant::now();
        let dt = instant
            .duration_since_epoch()
            .saturating_sub(n.duration_since_epoch());
        delay_cycles(dt.ticks());
    }
}

/// NOTE: I think this might be slower than using ::after(),
/// because it may not perform compile-time calculation of cycle count.
/// But either way this is useful for compatibility :)
impl DelayNs for Timer {
    fn delay_ns(&mut self, ns: u32) {
        Timer::after(Duration::from_nanos(ns as u64))
    }

    fn delay_us(&mut self, us: u32) {
        Timer::after(Duration::from_micros(us as u64))
    }

    fn delay_ms(&mut self, ms: u32) {
        Timer::after(Duration::from_millis(ms as u64))
    }
}
