//! LED matrix display implementation

use embedded_hal::{delay::DelayNs, spi::SpiDevice};

use crate::{
    Error, Max7219,
    led_matrix::{
        buffer::MatrixBuffer,
        fonts::{self, LedFont, STANDARD_LED_FONT},
        scroll::{ScrollConfig, ScrollingText},
    },
    registers::Digit,
};

/// A high-level abstraction for controlling an LED matrix display using the MAX7219 driver.
pub struct LedMatrix<SPI> {
    driver: Max7219<SPI>,
    default_font: &'static LedFont,
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
        Self {
            driver,
            default_font: &fonts::STANDARD_LED_FONT,
        }
    }

    /// Sets a custom default font for rendering characters on the LED matrix.
    ///
    /// This replaces the built-in default font with the provided one.
    /// The font will be used in all drawing operations that do not explicitly specify a font.
    ///
    /// This method follows the builder pattern and can be chained with other setup methods.
    ///
    /// # Arguments
    /// * `font` - A reference to a static `LedFont` instance to use as the default.
    ///
    /// # Returns
    /// Updated instance of `LedMatrix` with the new font set.
    pub fn with_font(mut self, font: &'static LedFont) -> Self {
        self.default_font = font;
        self
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
        Ok(Self {
            driver,
            default_font: &STANDARD_LED_FONT,
        })
    }

    /// Provides mutable access to the underlying MAX7219 driver.
    ///
    /// This allows users to call low-level functions directly
    pub fn driver(&mut self) -> &mut Max7219<SPI> {
        &mut self.driver
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
    pub fn draw_char(&mut self, device_index: usize, ch: char) -> Result<(), Error<SPI::Error>> {
        self.draw_char_with_font(device_index, ch, self.default_font)
    }

    /// Draws a single 8x8 character on the specified display device using a provided font.
    ///
    /// This function is similar to [`draw_char`], but allows overriding the font used for rendering.
    /// The character is mapped to an 8-byte bitmap. Each byte represents a row on the matrix, with
    /// the most significant bit (bit 7) on the left and the least significant bit (bit 0) on the right.
    ///
    /// # Arguments
    /// * `device_index` - Index of the MAX7219 device to write to.
    /// * `ch` - The character to render on the display.
    /// * `font` - The font to use for character lookup and rendering.
    ///
    pub fn draw_char_with_font(
        &mut self,
        device_index: usize,
        ch: char,
        font: &LedFont,
    ) -> Result<(), Error<SPI::Error>> {
        let bitmap = font.get_char(ch);
        // self.driver.draw_bitmap(bitmap, pos);
        for (row, value) in bitmap.iter().enumerate() {
            let digit_register = Digit::try_from(row as u8)?;
            self.driver
                .write_raw_digit(device_index, digit_register, *value)?;
        }
        Ok(())
    }

    /// Draw a string of text on the LED matrix using the default font.
    /// Each character is displayed on one device in the daisy chain.
    /// If the string is longer than the number of devices, the extra characters are ignored.
    pub fn draw_text(&mut self, text: &str) -> Result<(), Error<SPI::Error>> {
        self.draw_text_with_font(text, self.default_font)
    }

    /// Draw a string of text on the LED matrix using a specified font.
    /// Each character is displayed on one device in the daisy chain.
    /// If the string is longer than the number of devices, the extra characters are ignored.
    pub fn draw_text_with_font(
        &mut self,
        text: &str,
        font: &LedFont,
    ) -> Result<(), Error<SPI::Error>> {
        text.chars()
            .take(self.driver.device_count())
            .enumerate()
            .try_for_each(|(device_index, ch)| self.draw_char_with_font(device_index, ch, font))?;

        Ok(())
    }

    /// Scroll the given text across the LED matrix.
    ///
    /// This will render `text` using the current font and step through
    /// each frame at the delay specified by `config.step_delay_ns`. If
    /// `config.loop_text` is true, the text will repeat with
    /// `config.loop_padding` pixels of blank space between repetitions.
    ///
    ///
    /// # Parameters
    ///
    /// - `delay`: delay provider implementing `embedded_hal::delay::DelayNs`.
    /// - `text`: the string slice to scroll.
    /// - `config`: scrolling configuration (speed, step size, looping).
    ///
    /// # Errors
    ///
    /// Returns a `MatrixError` if updating the display buffer fails.
    pub fn scroll_text<D: DelayNs>(
        &mut self,
        delay: &mut D,
        text: &str,
        config: ScrollConfig,
    ) -> Result<(), Error<SPI::Error>> {
        let mut scroller = ScrollingText::new(text, self.default_font, config);
        scroller.reset();

        let device_count = self.driver().device_count();

        loop {
            // Update each display device
            for device_index in 0..device_count {
                let frame = scroller.get_frame()?; // Each device shows 8 pixels width
                self.write_buffer(device_index, &frame)?;

                // Advance offset for next device to create continuous scrolling
                scroller.current_offset += 8;
            }

            // Reset offset and step for next frame
            scroller.current_offset -= (device_count * 8) as i32;
            if !scroller.step() {
                break; // Stop if not looping and text has finished scrolling
            }

            delay.delay_ns(config.step_delay_ns);
        }

        Ok(())
    }

    /// Scroll the given text across the LED matrix using the default scroll configuration.
    pub fn scroll_text_default<D: DelayNs>(
        &mut self,
        delay: &mut D,
        text: &str,
    ) -> Result<(), Error<SPI::Error>> {
        self.scroll_text(delay, text, ScrollConfig::default())
    }
}
