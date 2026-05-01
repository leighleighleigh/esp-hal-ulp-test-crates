// Utility to measure a shared memory counter quickly in a loop,
// and try to estimate the frequency of counter updates.

use esp_hal::time::{Duration, Instant, Rate};

// Where is the counter placed in memory
pub const ADDRESS: usize = 0x5000_1000;
pub const DEBUG_ADDRESS: usize = 0x5000_1004;

#[inline]
pub fn reg_read(addr: usize) -> u32 {
    unsafe {
        let counter = addr as *mut u32;
        counter.read_volatile()
    }
}

#[inline]
pub fn reg_write(addr: usize, val: u32) {
    unsafe {
        let counter = addr as *mut u32;
        counter.write_volatile(val);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub count: u32,
    pub timestamp: Instant,
}

impl Sample {
    pub fn new() -> Self {
        let now = Instant::now();
        let c = reg_read(ADDRESS);
        Self {
            count: c,
            timestamp: now,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampleTachometer {
    last_sample: Option<Sample>,
    period_samples: u64,
    period_sum: u64,
    new_sample: bool,
}

impl SampleTachometer {
    pub fn new() -> Self {
        Self {
            last_sample: None,
            period_samples: 0,
            period_sum: 0,
            new_sample: true,
        }
    }

    pub fn reset(&mut self) {
        self.last_sample = None;
        self.period_samples = 0;
        self.period_sum = 0;
        self.new_sample = false;
    }

    pub fn changed(&mut self) -> bool {
        let c = self.new_sample;
        self.new_sample = false;
        c
    }

    pub fn last_sample(&self) -> Option<Sample> {
        self.last_sample
    }

    pub fn update(&mut self, new_sample: Sample) {
        // let new_sample = Sample::new();

        if let Some(old_sample) = &self.last_sample {
            if new_sample.count != old_sample.count {
                let count_change: u32 = new_sample.count - old_sample.count;
                let time_change = new_sample.timestamp - old_sample.timestamp;

                // Calculate microseconds difference
                let dt_us = time_change.as_micros();
                // calculate usec per count
                let count_period_us = dt_us / (count_change as u64);

                // Increment sample count
                self.period_samples += 1;
                self.period_sum += count_period_us;

                // Update changed flag
                self.new_sample = true;
            }
        } else {
            self.new_sample = true;
        }

        // Save the new values over the last count / last time
        if self.new_sample {
            self.last_sample = Some(new_sample);
        }
    }

    pub fn count_rate(&self) -> Option<Rate> {
        // divide the total period sum by sample count
        match self.count_period() {
            Some(period) => Some(Rate::from_hz((1_000_000 / period.as_micros()) as u32)),
            None => None,
        }
    }

    pub fn count_period(&self) -> Option<Duration> {
        if self.period_samples == 0 {
            return None;
        }
        // divide the total period sum by sample count
        let avg_period_us = self.period_sum / self.period_samples;
        Some(Duration::from_micros(avg_period_us))
    }
}
