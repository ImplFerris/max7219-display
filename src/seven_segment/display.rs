//! 7-segment display implementation

use embedded_hal::spi::SpiDevice;

use crate::{Error, Max7219, seven_segment::Font};

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
    /// * `display_count` - Number of daisy-chained MAX7219 displays.
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
    pub fn from_spi(spi: SPI, display_count: usize) -> Result<Self, Error<SPI::Error>> {
        let mut driver = Max7219::new(spi).with_device_count(display_count)?;
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
    pub fn write_char(
        &mut self,
        digit: u8,
        ch: char,
        font: &Font,
    ) -> Result<(), Error<SPI::Error>> {
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
    ) -> Result<(), Error<SPI::Error>> {
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
    pub fn write_bcd_char(&mut self, digit: u8, ch: char) -> Result<(), Error<SPI::Error>> {
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
