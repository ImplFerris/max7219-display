//! LED matrix display implementation

use embedded_hal::spi::SpiDevice;

use crate::{
    Max7219,
    error::Error,
    led_matrix::{
        buffer::MatrixBuffer,
        fonts::{FONT8X8_UNKNOWN, get_char_bitmap},
    },
    registers::Digit,
};

/// A high-level abstraction for controlling an LED matrix display using the MAX7219 driver.
pub struct LedMatrix<SPI> {
    driver: Max7219<SPI>,
}

impl<SPI> LedMatrix<SPI>
where
    SPI: SpiDevice,
{
    /// Creates a new `LedMatrix` instance from an existing `Max7219` driver.
    ///
    /// # Arguments
    ///
    /// * `driver` - An initialized `Max7219` driver instance.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let driver = Max7219::new(spi).with_device_count(1).expect("device count 1 should not panic");
    /// let mut matrix = LedMatrix::new(driver);
    /// ```
    pub fn new(driver: Max7219<SPI>) -> Self {
        Self { driver }
    }

    /// Simplifies initialization by creating a new `LedMatrix` instance
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
    /// Returns a `LedMatrix` instance on success, or an error if the display count is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let spi = /* your SPI device */;
    /// let mut matrix = LedMatrix::from_spi(spi, 4).unwrap();
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

    /// Draws a single 8x8 character on the specified display device.
    ///
    /// The character is converted into an 8-byte bitmap using a predefined font.
    /// If the character is unsupported, it will be replaced with "?" char.
    ///
    /// Each byte in the bitmap corresponds to one row of the 8x8 LED matrix (from D0 to D7),
    /// and is written to the digit registers of the specified `device_index`.
    ///
    /// # Arguments
    /// * `device_index` - Index of the target MAX7219 device in the daisy chain.
    /// * `c` - The character to display.
    ///
    /// # Errors
    /// Returns an error if the digit conversion fails or if SPI communication fails.
    pub fn draw_char(&mut self, device_index: usize, c: char) -> Result<(), Error<SPI::Error>> {
        let bitmap = get_char_bitmap(c).unwrap_or(FONT8X8_UNKNOWN);

        for (row, value) in bitmap.iter().enumerate() {
            let digit_register = Digit::try_from(row as u8)?;
            self.driver
                .write_raw_digit(device_index, digit_register, *value)?;
        }
        Ok(())
    }

    /// Clear a specific device
    pub fn clear(&mut self, device_index: usize) -> Result<(), Error<SPI::Error>> {
        self.driver.clear_display(device_index)
    }

    /// Write a complete buffer to a specific display
    pub fn write_buffer(
        &mut self,
        device_index: usize,
        buffer: &MatrixBuffer,
    ) -> Result<(), Error<SPI::Error>> {
        for (row, &data) in buffer.data().iter().enumerate() {
            let digit = Digit::try_from(row as u8)?;
            self.driver.write_raw_digit(device_index, digit, data)?;
        }
        Ok(())
    }
}
