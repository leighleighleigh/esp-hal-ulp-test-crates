use super::traits::SharedCell;

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

pub type UlpCmdWrapper = SharedCell<UlpCmd>;

impl UlpCmdWrapper {
    pub const fn new() -> Self {
        Self::init(UlpCmd::UNSET)
    }
}

pub type UlpReplyWrapper = SharedCell<UlpReply>;

impl UlpReplyWrapper {
    pub const fn new() -> Self {
        Self::init(UlpReply::UNSET)
    }
}

pub type UlpCounter = SharedCell<u32>;

impl UlpCounter {
    pub const fn new() -> Self {
        Self::init(0)
    }

    /// Increments the counter in-place.
    pub fn increment(&self) {
        let c = self.load() + 1;
        self.store(c);
    }

    // Reset the counter to 0
    pub fn reset(&self) {
        self.store(0);
    }
}
