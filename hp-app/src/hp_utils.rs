use esp32s3 as pac;

/// Configures the RTC GPIO pins so they can be used by the LP core,
/// and enables interrupts for them.
/// int_type: 1 = rising, 2 = falling, 3 = any, 4 = low, 5 = high
pub fn configure_rtc_pin(pin: usize, int_type: u8, wakeup: bool, open_drain: bool) {
    let rtc_regs = unsafe { &*pac::RTC_IO::PTR };

    // Set the following bits for the pin pad
    // mux_sel - 1, enabled to route pin to RTC peripheral
    // fun_ie - 1, enable input mode
    // rue - 0, disable pullup
    // rde - 0, disable pulldown
    // fun_sel - 0, normal function

    // I'm using a macro here to avoid heaps of repetition across the pads.
    macro_rules! configure_pad {
        ($expression:expr) => {{
            $expression.write(|w| unsafe {
                w.mux_sel()
                    .set_bit()
                    .fun_ie()
                    .set_bit()
                    .rue()
                    .clear_bit()
                    .rde()
                    .clear_bit()
                    .fun_sel()
                    .bits(0)
                    .slp_ie()
                    .set_bit()
                    .slp_sel() // Enable sleep mode
                    .set_bit()
            });
            // Return nothing
            ()
        }};
    }

    match pin {
        // These IO pins do not require any special treatment
        n if (n <= 14) || (n == 21) => {
            configure_pad!(rtc_regs.touch_pad(n))
        }
        // These pins are different, as they are hooked to an XTAL oscillator
        15 => configure_pad!(rtc_regs.xtal_32p_pad()),
        16 => configure_pad!(rtc_regs.xtal_32n_pad()),
        17 => configure_pad!(rtc_regs.pad_dac1()),
        18 => configure_pad!(rtc_regs.pad_dac2()),
        // Pins 19 and 20 are connected to USB D- and USB D+, respectively.
        19 => configure_pad!(rtc_regs.rtc_pad19()),
        20 => configure_pad!(rtc_regs.rtc_pad20()),
        _ => {}
    };

    // Enable the interrupt and wakeup modes for the pin
    rtc_regs.pin(pin).write(|w| unsafe {
        w.int_type()
            .bits(int_type)
            .wakeup_enable()
            .variant(wakeup)
            .pad_driver()
            .variant(open_drain)
    });
}
