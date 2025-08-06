//! MAX7219 register definitions and constants

use crate::{Error, Result};

/// MAX7219 control register addresses.
///
/// Each variant corresponds to a register in the MAX7219 display driver chip.
/// These registers are used to control display behavior, including digit data,
/// decoding mode, brightness, scan limit, power state, and test mode.
///
/// This enum is typically used when sending 16-bit data packets to the MAX7219,
/// where the upper byte specifies the target register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    /// No-op register
    NoOp = 0x00,
    /// Digit 0 register
    Digit0 = 0x01,
    /// Digit 1 register
    Digit1 = 0x02,
    /// Digit 2 register
    Digit2 = 0x03,
    /// Digit 3 register
    Digit3 = 0x04,
    /// Digit 4 register
    Digit4 = 0x05,
    /// Digit 5 register
    Digit5 = 0x06,
    /// Digit 6 register
    Digit6 = 0x07,
    /// Digit 7 register
    Digit7 = 0x08,
    /// Decode mode register
    DecodeMode = 0x09,
    /// Intensity register
    Intensity = 0x0A,
    /// Scan limit register
    ScanLimit = 0x0B,
    /// Shutdown register
    Shutdown = 0x0C,
    /// Display test register
    DisplayTest = 0x0F,
}

impl Register {
    /// Convert register to u8 value
    pub const fn addr(self) -> u8 {
        self as u8
    }

    /// Try to convert a digit index (0-7) into a corresponding `Register::DigitN`.
    pub(crate) fn try_digit(digit: u8) -> Result<Self> {
        match digit {
            0 => Ok(Register::Digit0),
            1 => Ok(Register::Digit1),
            2 => Ok(Register::Digit2),
            3 => Ok(Register::Digit3),
            4 => Ok(Register::Digit4),
            5 => Ok(Register::Digit5),
            6 => Ok(Register::Digit6),
            7 => Ok(Register::Digit7),
            _ => Err(Error::InvalidDigit),
        }
    }

    /// Returns an iterator over all digit registers (Digit0 to Digit7).
    ///
    /// Useful for iterating through display rows or columns when writing
    /// to all digits of a MAX7219 device in order.
    pub fn digits() -> impl Iterator<Item = Register> {
        [
            Register::Digit0,
            Register::Digit1,
            Register::Digit2,
            Register::Digit3,
            Register::Digit4,
            Register::Digit5,
            Register::Digit6,
            Register::Digit7,
        ]
        .into_iter()
    }
}

/// Decode mode configuration for the MAX7219 display driver.
///
/// Code B decoding allows the driver to automatically convert certain values
/// (such as 0-9, E, H, L, and others) into their corresponding 7-segment patterns.
/// Digits not using Code B must be controlled manually using raw segment data.
///
/// Use this to configure which digits should use Code B decoding and which
/// should remain in raw segment mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DecodeMode {
    /// Disable Code B decoding for all digits (DIG0 to DIG7).
    ///
    /// In this mode, you must manually set each segment (A to G and DP)
    /// using raw segment data.
    NoDecode = 0x00,

    /// Enable Code B decoding for only digit 0 (DIG0).
    ///
    /// All other digits (DIG1 to DIG7) must be controlled manually.
    Digit0 = 0x01,

    /// Enable Code B decoding for digits 0 through 3 (DIG0 to DIG3).
    ///
    /// This is commonly used for 4-digit numeric displays.
    Digits0To3 = 0x0F,

    /// Enable Code B decoding for all digits (DIG0 to DIG7).
    ///
    /// This is typically used for full 8-digit numeric displays.
    AllDigits = 0xFF,
}

impl DecodeMode {
    /// Convert decode mode to u8 value
    pub const fn value(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_addr() {
        assert_eq!(Register::NoOp.addr(), 0x00);
        assert_eq!(Register::Digit0.addr(), 0x01);
        assert_eq!(Register::Digit7.addr(), 0x08);
        assert_eq!(Register::DecodeMode.addr(), 0x09);
        assert_eq!(Register::Intensity.addr(), 0x0A);
        assert_eq!(Register::ScanLimit.addr(), 0x0B);
        assert_eq!(Register::Shutdown.addr(), 0x0C);
        assert_eq!(Register::DisplayTest.addr(), 0x0F);
    }

    #[test]
    fn test_try_digit_valid() {
        assert_eq!(Register::try_digit(0), Ok(Register::Digit0));
        assert_eq!(Register::try_digit(1), Ok(Register::Digit1));
        assert_eq!(Register::try_digit(2), Ok(Register::Digit2));
        assert_eq!(Register::try_digit(3), Ok(Register::Digit3));
        assert_eq!(Register::try_digit(4), Ok(Register::Digit4));
        assert_eq!(Register::try_digit(5), Ok(Register::Digit5));
        assert_eq!(Register::try_digit(6), Ok(Register::Digit6));
        assert_eq!(Register::try_digit(7), Ok(Register::Digit7));
    }

    #[test]
    fn test_try_digit_invalid() {
        assert_eq!(Register::try_digit(8), Err(Error::InvalidDigit));
        assert_eq!(Register::try_digit(255), Err(Error::InvalidDigit));
    }

    #[test]
    fn test_digits_iterator() {
        let expected = [
            Register::Digit0,
            Register::Digit1,
            Register::Digit2,
            Register::Digit3,
            Register::Digit4,
            Register::Digit5,
            Register::Digit6,
            Register::Digit7,
        ];
        let actual: Vec<Register> = Register::digits().collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decode_mode_value() {
        assert_eq!(DecodeMode::NoDecode.value(), 0x00);
        assert_eq!(DecodeMode::Digit0.value(), 0x01);
        assert_eq!(DecodeMode::Digits0To3.value(), 0x0F);
        assert_eq!(DecodeMode::AllDigits.value(), 0xFF);
    }
}
