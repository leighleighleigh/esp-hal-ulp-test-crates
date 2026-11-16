use super::traits::SharedWrapper;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpCmd {
    UNSET                = 0,
    NOOP                 = 1,
    COUNTER_ONESHOT      = 2,
    COUNTER_LOOP         = 3,
    COUNTER_ULP_TIMER    = 4,
    XOR_TEST             = 5,
    STOP_TEST            = 6,
    MUTEX_TEST           = 7,
    TIMER_PERIOD_TEST    = 8,
    LIGHT_SLEEP_TEST     = 9,
    EXCEPTION_TEST       = 10,
    START_INT_TEST       = 11,
    WAITIRQ_TIMER_TEST   = 12,
    GPIO_INT_TEST        = 13,
    GPIO_WAKEUP_TEST     = 14,
    MAX1708_I2C_BMS_TEST = 15,
    WS2812_LED_TEST      = 16,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(not(feature = "is-lp-core"), derive(Debug))]
#[derive(Clone, Copy, PartialOrd, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum UlpReply {
    UNSET = 0,
    OK    = 1,
    NOK   = 2,
    BUSY  = 3,
}

pub type UlpCmdWrapper = SharedWrapper<UlpCmd>;
pub type UlpReplyWrapper = SharedWrapper<UlpReply>;
pub type UlpCounter = SharedWrapper<u32>;

impl UlpCounter {
    // Increments counter and returns the new value
    pub fn increment(&self) -> u32 {
        let c = self.load() + 1;
        self.store(c);
        c
    }

    pub fn reset(&self) {
        self.store(0);
    }
}
