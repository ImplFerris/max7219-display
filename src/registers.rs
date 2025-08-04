//! MAX7219 register definitions and constants

use crate::error::DigitError;

/// MAX7219 register addresses.
///
/// Each variant in this enum represents one of the control registers
/// inside the MAX7219 display driver chip. These registers are used
/// to configure various display settings and control individual digits.
///
/// This enum is typically used when writing 16-bit data packets to the MAX7219.
/// The address byte (bits D8–D11) selects which register to write to.
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
    pub(crate) fn try_digit(digit: u8) -> Result<Self, DigitError> {
        match digit {
            0 => Ok(Register::Digit0),
            1 => Ok(Register::Digit1),
            2 => Ok(Register::Digit2),
            3 => Ok(Register::Digit3),
            4 => Ok(Register::Digit4),
            5 => Ok(Register::Digit5),
            6 => Ok(Register::Digit6),
            7 => Ok(Register::Digit7),
            _ => Err(DigitError::InvalidDigit),
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

/// Represents the decode mode for the MAX7219 display driver.
///
/// The decode mode determines which digits use "Code B" decoding.
/// In Code B mode, the driver automatically maps numeric values (0–9, E, H, L, etc.)
/// to their corresponding 7-segment patterns. Digits not using Code B must be
/// manually controlled by setting individual segments.
///
/// You can choose to enable Code B for specific digits or disable it entirely.
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
