//! 7-segment display implementation

use embedded_hal::spi::SpiDevice;

use crate::{Error, Max7219, Result, seven_segment::Font};

/// A high-level abstraction for controlling a 7-segment display using the MAX7219 driver.
pub struct SevenSegment<SPI> {
    driver: Max7219<SPI>,
}

impl<SPI> SevenSegment<SPI>
where
    SPI: SpiDevice,
{
    /// Creates a new `SevenSegment` instance from an existing `Max7219` driver.
    ///
    /// # Arguments
    ///
    /// * `driver` - An initialized `Max7219` driver instance.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let driver = Max7219::new(spi)
    ///     .with_device_count(2)
    ///     .expect("2 is valid device count");
    /// let mut display = SevenSegment::new(driver);
    /// ```
    pub fn new(driver: Max7219<SPI>) -> Self {
        Self { driver }
    }

    /// Simplifies initialization by creating a new `SevenSegment` instance
    /// from the given SPI device and number of connected displays.
    ///
    /// Internally, this constructs and initializes the `Max7219` driver,
    /// making setup easier for typical use cases.
    ///
    /// # Arguments
    ///
    /// * `spi` - The SPI device used for communication.
    /// * `device_count` - Number of daisy-chained MAX7219 devices.
    ///
    /// # Returns
    ///
    /// Returns a `SevenSegment` instance on success, or an error if the display count is invalid
    /// or if the MAX7219 initialization fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let spi = /* your SPI device */;
    /// let mut display = SevenSegment::from_spi(spi, 4).unwrap();
    /// ```
    pub fn from_spi(spi: SPI, device_count: usize) -> Result<Self> {
        let mut driver = Max7219::new(spi).with_device_count(device_count)?;
        driver.init()?;
        Ok(Self { driver })
    }

    /// Provides mutable access to the underlying MAX7219 driver.
    ///
    /// This allows users to call low-level functions directly
    pub fn driver(&mut self) -> &mut Max7219<SPI> {
        &mut self.driver
    }

    /// Writes a character to a specific 7-segment display (i.e., a `Digit`) on the first MAX7219 device.
    ///
    /// This is a convenience method for single-device setups.
    /// Converts the character to a 7-segment pattern using the provided font,
    /// and writes it to the specified digit.
    ///
    /// ```text
    /// Segment layout of single digit:
    ///
    ///      A
    ///     ---
    ///  F |   | B
    ///     ---    ← G (middle)
    ///  E |   | C
    ///     ---   
    ///      D       • DP
    ///
    /// The MAX7219 can control up to 8 seven-segment displays (from Digit0 to Digit7).
    /// Each 7-segment display corresponds to a digit register.
    ///
    /// ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
    /// │  0  │  1  │  2  │  3  │  4  │  5  │  6  │  7  │ ← Digit positions
    /// └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘
    ///
    /// Example: write_char(Digit::D3, 'A', &font) writes 'A' to position 3:
    ///
    /// Character 'A' = 0b01110111 (DP G F E D C B A)
    ///                              0 1 1 1 0 1 1 1
    ///
    /// ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
    /// │     │     │     │ --- │     │     │     │     │ ← A = 1 (top)
    /// │     │     │     │|   |│     │     │     │     │ ← F = 1, B = 1 (sides)
    /// │     │     │     │ --- │     │     │     │     │ ← G = 1 (middle)
    /// │     │     │     │|   |│     │     │     │     │ ← E = 1, C = 1 (sides)
    /// │     │     │     │     │     │     │     │     │ ← D = 0 (bottom off)
    /// └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘
    ///                     ↑
    ///                 Position 3
    ///
    /// ```
    ///
    /// # Arguments
    ///
    /// * `digit` - The digit position (0 to 7) on the device.
    /// * `ch` - The character to write.
    /// * `font` - The font used to convert the character into a segment pattern
    pub fn write_char(&mut self, digit: u8, ch: char, font: &Font) -> Result<()> {
        self.write_char_to_device(0, digit, ch, font)
    }

    /// Writes a character to a specific digit on a specific MAX7219 device.
    ///
    /// Converts the given character into a 7-segment segment pattern using the provided font,
    /// and writes it to the specified digit position on the selected device in the daisy chain.
    ///
    /// # Arguments
    ///
    /// * `device_index` - The index of the target device (0 for the first device).
    /// * `digit` - The digit position (0 to 7) on the device.
    /// * `ch` - The character to write.
    /// * `font` - The font used to convert the character into a segment pattern.
    pub fn write_char_to_device(
        &mut self,
        device_index: usize,
        digit: u8,
        ch: char,
        font: &Font,
    ) -> Result<()> {
        let data = font.get_char(ch);

        self.driver.write_raw_digit(device_index, digit, data)?;

        Ok(())
    }

    /// Writes a BCD-compatible character to a digit on the first MAX7219 device.
    ///
    /// This method assumes the MAX7219 is configured in BCD decode mode
    /// (`DecodeMode::CodeB`). Only a limited set of characters are supported
    /// in this mode: digits '0'..='9', 'E', 'H', 'L', 'P', '-' and blank (space).
    ///
    /// The BCD decode mode must be enabled beforehand using `set_decode_mode()`.
    ///
    /// Returns an error if the character is not supported in BCD mode.
    pub fn write_bcd_char(&mut self, digit: u8, ch: char) -> Result<()> {
        let data = match ch {
            '0'..='9' => ch as u8 - b'0',
            '-' => 0x0A,
            'E' => 0x0B,
            'H' => 0x0C,
            'L' => 0x0D,
            'P' => 0x0E,
            ' ' => 0x0F,
            _ => return Err(Error::UnsupportedChar),
        };

        self.driver.write_raw_digit(0, digit, data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, Max7219, Register,
        seven_segment::{STANDARD_FONT, SevenSegment, fonts},
    };
    use embedded_hal_mock::eh1::{spi::Mock as SpiMock, spi::Transaction};

    #[test]
    fn test_from_spi() {
        let device_count = 2;
        // Expected transactions for Max7219::init() on 2 devices
        let expected_transactions = vec![
            // power_on (2 devices)
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Shutdown.addr(),
                0x01,
                Register::Shutdown.addr(),
                0x01,
            ]),
            Transaction::transaction_end(),
            // test_all(false) (2 devices)
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::DisplayTest.addr(),
                0x00,
                Register::DisplayTest.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // set_scan_limit_all(NUM_DIGITS) (2 devices)
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::ScanLimit.addr(),
                crate::NUM_DIGITS - 1,
                Register::ScanLimit.addr(),
                crate::NUM_DIGITS - 1,
            ]),
            Transaction::transaction_end(),
            // set_decode_mode_all(NoDecode) (2 devices)
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::DecodeMode.addr(),
                crate::registers::DecodeMode::NoDecode as u8,
                Register::DecodeMode.addr(),
                crate::registers::DecodeMode::NoDecode as u8,
            ]),
            Transaction::transaction_end(),
            // clear_all() - 8 transactions for 8 digits, each affecting 2 devices
            // Digit0
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit0.addr(),
                0x00,
                Register::Digit0.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // Digit1
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit1.addr(),
                0x00,
                Register::Digit1.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // Digit2
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit2.addr(),
                0x00,
                Register::Digit2.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // Digit3
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit3.addr(),
                0x00,
                Register::Digit3.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // Digit4
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit4.addr(),
                0x00,
                Register::Digit4.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // Digit5
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit5.addr(),
                0x00,
                Register::Digit5.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // Digit6
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit6.addr(),
                0x00,
                Register::Digit6.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
            // Digit7
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::Digit7.addr(),
                0x00,
                Register::Digit7.addr(),
                0x00,
            ]),
            Transaction::transaction_end(),
        ];

        let mut spi = SpiMock::new(&expected_transactions);
        let result = SevenSegment::from_spi(&mut spi, device_count);

        assert!(result.is_ok());

        spi.done();
    }

    #[test]
    fn test_from_spi_invalid_count() {
        let mut spi = SpiMock::new(&[]); // No SPI calls expected if count is invalid
        let result = SevenSegment::from_spi(&mut spi, crate::MAX_DISPLAYS + 1);

        assert!(matches!(result, Err(Error::InvalidDeviceCount)));

        spi.done();
    }

    #[test]
    fn test_write_char() {
        let ch = 'R';
        let digit = 2;
        let font = fonts::STANDARD_FONT;
        let expected_data = font.get_char(ch);

        // Expected transaction for writing to digit 2 on device 0
        let expected_transactions = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![Register::Digit2.addr(), expected_data]),
            Transaction::transaction_end(),
        ];

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut display = SevenSegment::new(driver);

        let result = display.write_char(digit, ch, &font);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_write_char_to_device() {
        let device_index = 1;
        let ch = '3';
        let digit = 5;
        let font = fonts::STANDARD_FONT;
        let expected_data = font.get_char(ch);

        let expected_transactions = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![
                Register::NoOp.addr(),
                0x00,
                Register::Digit5.addr(),
                expected_data,
            ]),
            Transaction::transaction_end(),
        ];

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi).with_device_count(2).unwrap();
        let mut display = SevenSegment::new(driver);

        let result = display.write_char_to_device(device_index, digit, ch, &font);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_write_char_to_device_invalid_index() {
        let mut spi = SpiMock::new(&[]); // No SPI calls expected
        let driver = Max7219::new(&mut spi).with_device_count(1).unwrap();
        let mut display = SevenSegment::new(driver);

        let result = display.write_char_to_device(1, 0, 'A', &STANDARD_FONT); // Index 1 is invalid for device_count=1
        assert_eq!(result, Err(Error::InvalidDeviceIndex));
        spi.done();
    }

    #[test]
    fn test_write_char_invalid_digit() {
        let mut spi = SpiMock::new(&[]); // No SPI calls expected
        let driver = Max7219::new(&mut spi);
        let mut display = SevenSegment::new(driver);

        let result = display.write_char(8, 'A', &STANDARD_FONT); // Digit 8 is invalid
        // This will fail inside Max7219::write_raw_digit -> Register::try_digit
        assert_eq!(result, Err(Error::InvalidDigit));
        spi.done();
    }

    #[test]
    fn test_write_bcd_char() {
        let ch = '5';
        let digit = 3;
        let expected_data = 0x05; // BCD for '5'

        let expected_transactions = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![Register::Digit3.addr(), expected_data]),
            Transaction::transaction_end(),
        ];

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut display = SevenSegment::new(driver);

        let result = display.write_bcd_char(digit, ch);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_write_bcd_char_dash() {
        let ch = '-';
        let digit = 1;
        let expected_data = 0x0A;

        let expected_transactions = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![Register::Digit1.addr(), expected_data]),
            Transaction::transaction_end(),
        ];

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut display = SevenSegment::new(driver);

        let result = display.write_bcd_char(digit, ch);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_write_bcd_char_unsupported() {
        let mut spi = SpiMock::new(&[]); // No SPI calls expected
        let driver = Max7219::new(&mut spi);
        let mut display = SevenSegment::new(driver);

        let result = display.write_bcd_char(0, 'X'); // 'X' is not supported in BCD mode
        assert_eq!(result, Err(Error::UnsupportedChar));
        spi.done();
    }

    #[test]
    fn test_write_bcd_char_invalid_digit() {
        let mut spi = SpiMock::new(&[]); // No SPI calls expected
        let driver = Max7219::new(&mut spi);
        let mut display = SevenSegment::new(driver);

        let result = display.write_bcd_char(8, '0'); // Digit 8 is invalid
        // This will fail inside Max7219::write_raw_digit -> Register::try_digit
        assert_eq!(result, Err(Error::InvalidDigit));
        spi.done();
    }

    // Test driver() method indirectly by using it to call a Max7219 function
    #[test]
    fn test_driver_mut_access() {
        let expected_transactions = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![Register::Shutdown.addr(), 0x01]),
            Transaction::transaction_end(),
        ];
        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut display = SevenSegment::new(driver);

        // Use the driver() method to access the underlying driver and call power_on
        let result = display.driver().power_on();
        assert!(result.is_ok());

        spi.done();
    }
}
