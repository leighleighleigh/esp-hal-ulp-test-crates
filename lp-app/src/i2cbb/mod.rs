//! # Synchronous implementation of embedded-hal I2C traits based on GPIO bitbang
//!
//! This implementation consumes the following hardware resources:
//! - A periodic timer to mark clock cycles
//! - Two GPIO pins for SDA and SCL lines.
//!
//! Note that the current implementation does not support I2C clock stretching.
//!
//! ## Hardware requirements
//!
//! 1. Configure GPIO pins as Open-Drain outputs.
//! 2. Configure timer frequency to be twice the desired I2C clock frequency.

// use crate::delay::{Timer,BlockingTimer,Ticker,BlockingTicker};
// use crate::delay::BlockingTimer as DelayNs;

use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    i2c::{ErrorKind, ErrorType, I2c, NoAcknowledgeSource, Operation},
};

/// Bit banging I2C device
pub struct I2cBB<SCL, SDA, CLK>
where
    SCL: OutputPin,
    SDA: OutputPin + InputPin,
    CLK: DelayNs,
{
    scl: SCL,
    sda: SDA,
    clk: CLK,
}

impl<SCL, SDA, CLK, E> I2cBB<SCL, SDA, CLK>
where
    SCL: OutputPin<Error = E>,
    SDA: OutputPin<Error = E> + InputPin<Error = E>,
    CLK: DelayNs,
{
    /// Create instance
    pub fn new(scl: SCL, sda: SDA, clk: CLK) -> Self {
        I2cBB { scl, sda, clk }
    }

    /// Send a raw I2C start.
    ///
    /// **This is a low-level control function.** For normal I2C devices,
    /// please use the embedded-hal traits [Read], [Write], or
    /// [WriteRead].
    fn raw_i2c_start(&mut self) {
        self.set_scl_high();
        self.set_sda_high();
        self.wait_for_clk();
        self.set_sda_low();
        self.wait_for_clk();
        self.set_scl_low();
        self.wait_for_clk();
    }

    /// Send a raw I2C stop.
    ///
    /// **This is a low-level control function.** For normal I2C devices,
    /// please use the embedded-hal traits [Read], [Write], or
    /// [WriteRead].
    fn raw_i2c_stop(&mut self) {
        self.set_scl_high();
        self.wait_for_clk();
        self.set_sda_high();
        self.wait_for_clk();
    }

    fn i2c_is_ack(&mut self) -> Result<bool, ErrorKind> {
        self.set_sda_high();
        self.set_scl_high();
        self.wait_for_clk();
        let ack = self
            .sda
            .is_low()
            .map_err(|_| ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown))?;
        self.set_scl_low();
        self.set_sda_low();
        self.wait_for_clk();
        Ok(ack)
    }

    fn i2c_read_byte(&mut self, should_send_ack: bool) -> Result<u8, ErrorKind> {
        let mut byte: u8 = 0;

        self.set_sda_high();

        for bit_offset in 0..8 {
            self.set_scl_high();
            self.wait_for_clk();

            if self.sda.is_high().map_err(|_| ErrorKind::Other)? {
                byte |= 1 << (7 - bit_offset);
            }

            self.set_scl_low();
            self.wait_for_clk();
        }

        if should_send_ack {
            self.set_sda_low();
        } else {
            self.set_sda_high();
        }

        self.set_scl_high();
        self.wait_for_clk();

        self.set_scl_low();
        self.set_sda_low();
        self.wait_for_clk();

        Ok(byte)
    }

    fn i2c_write_byte(&mut self, byte: u8) -> Result<(), ErrorKind> {
        for bit_offset in 0..8 {
            let out_bit = (byte >> (7 - bit_offset)) & 0b1;
            if out_bit == 1 {
                self.set_sda_high();
            } else {
                self.set_sda_low();
            }
            self.set_scl_high();
            self.wait_for_clk();
            self.set_scl_low();
            self.set_sda_low();
            self.wait_for_clk();
        }

        Ok(())
    }

    /// Read raw bytes from the slave.
    ///
    /// **This is a low-level control function.** For normal I2C devices,
    /// please use the embedded-hal traits [Read], [Write], or
    /// [WriteRead].
    // #[inline]
    fn raw_read_from_slave(&mut self, input: &mut [u8]) -> Result<(), ErrorKind> {
        for i in 0..input.len() {
            let should_send_ack = i != (input.len() - 1);
            input[i] = self.i2c_read_byte(should_send_ack)?;
        }
        Ok(())
    }

    /// Send raw bytes to the slave.
    ///
    /// **This is a low-level control function.** For normal I2C devices,
    /// please use the embedded-hal traits [Read], [Write], or
    /// [WriteRead].
    // #[inline]
    fn raw_write_to_slave(&mut self, output: &[u8]) -> Result<(), ErrorKind> {
        for byte in output {
            self.i2c_write_byte(*byte)?;
            self.check_ack()
                .map_err(|_| ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data))?
        }
        Ok(())
    }

    fn set_scl_high(&mut self) {
        self.scl.set_high().ok();
    }

    fn set_scl_low(&mut self) {
        self.scl.set_low().ok();
    }

    fn set_sda_high(&mut self) {
        self.sda.set_high().ok();
    }

    fn set_sda_low(&mut self) {
        self.sda.set_low().ok();
    }

    fn wait_for_clk(&mut self) {
        // delay_us(1) worked perfectly :)
        // self.clk.delay_us(1); // ~100kHz

        // Im going to try and get moar speed though >:)
        // 400kHz - working omg!
        self.clk.delay_ns(250);

        // 800kHz...
        // self.clk.delay_ns(125);
    }

    // #[inline]
    fn check_ack(&mut self) -> Result<(), ErrorKind> {
        if !self.i2c_is_ack()? {
            Err(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown))
        } else {
            Ok(())
        }
    }
}

impl<SCL, SDA, CLK, E> ErrorType for I2cBB<SCL, SDA, CLK>
where
    SCL: OutputPin<Error = E>,
    SDA: OutputPin<Error = E> + InputPin<Error = E>,
    CLK: DelayNs,
{
    type Error = ErrorKind;
}

impl<SCL, SDA, CLK, E> I2c for I2cBB<SCL, SDA, CLK>
where
    SCL: OutputPin<Error = E>,
    SDA: OutputPin<Error = E> + InputPin<Error = E>,
    CLK: DelayNs,
{
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for op in operations {
            match op {
                Operation::Read(items) => {
                    if items.len() == 0 {
                        continue;
                    }
                    // ST
                    self.raw_i2c_start();
                    // SAD + R
                    self.i2c_write_byte((address << 1) | 0x1)?;
                    self.check_ack()
                        .map_err(|_| ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address))?;
                    self.raw_read_from_slave(items)?;
                    // SP
                    self.raw_i2c_stop();
                }
                Operation::Write(items) => {
                    self.raw_i2c_start();
                    self.i2c_write_byte((address << 1) | 0x0)?;
                    self.check_ack()
                        .map_err(|_| ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address))?;
                    self.raw_write_to_slave(items)?;
                    self.raw_i2c_stop();
                }
            }
        }
        Ok(())
    }
}
