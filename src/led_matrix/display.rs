//! LED matrix display implementation

use embedded_hal::{delay::DelayNs, spi::SpiDevice};

use crate::{
    Error, MAX_DISPLAYS, Max7219, Register,
    led_matrix::{
        buffer::MatrixBuffer,
        fonts::{self, LedFont},
        scroll::{ScrollConfig, ScrollingText},
    },
};

/// Represents a single 8x8 LED matrix controlled by one MAX7219 device.
pub type SingleMatrix<SPI> = LedMatrix<SPI, 64, 1>;

/// Represents a 4-in-1 LED matrix module (total 8x32 pixels) using four chained MAX7219 devices.
pub type Matrix4<SPI> = LedMatrix<SPI, 256, 4>;

/// Represents an 8-in-1 LED matrix module (total 8x64 pixels) using eight chained MAX7219 devices.
pub type Matrix8<SPI> = LedMatrix<SPI, 512, 8>;

/// A high-level abstraction for controlling an LED matrix display using the MAX7219 driver.
pub struct LedMatrix<SPI, const BUFFER_LENGTH: usize = 64, const DEVICE_COUNT: usize = 1> {
    driver: Max7219<SPI>,
    /// The framebuffer with one `u8` per pixel (0 = off, non-zero = on).
    ///
    /// Each 8x8 display has 64 pixels. For `N` daisy-chained devices,
    /// the total framebuffer size is `N * 64` pixels.
    ///
    /// For example, with 4 devices: `4 * 64 = 256` pixels.
    ///
    /// This buffer is modified by `embedded-graphics` through the
    /// [`DrawTarget`](https://docs.rs/embedded-graphics-core/latest/embedded_graphics_core/draw_target/trait.DrawTarget.html) trait.
    framebuffer: [u8; BUFFER_LENGTH],
}

impl<SPI, const BUFFER_LENGTH: usize, const DEVICE_COUNT: usize>
    LedMatrix<SPI, BUFFER_LENGTH, DEVICE_COUNT>
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
            framebuffer: [0; BUFFER_LENGTH],
        }
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
    pub fn from_spi(spi: SPI) -> Result<Self, Error<SPI::Error>> {
        let mut driver = Max7219::new(spi).with_device_count(DEVICE_COUNT)?;
        driver.init()?;
        Ok(Self {
            driver,
            framebuffer: [0; BUFFER_LENGTH],
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

    /// Clear all device
    pub fn clear_all(&mut self) -> Result<(), Error<SPI::Error>> {
        self.driver.clear_all()
    }

    /// Write a complete buffer to a specific display
    pub fn write_buffer(
        &mut self,
        device_index: usize,
        buffer: &MatrixBuffer,
    ) -> Result<(), Error<SPI::Error>> {
        for (row, &data) in buffer.data().iter().enumerate() {
            self.driver.write_raw_digit(device_index, row as u8, data)?;
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
        self.draw_char_with_font(device_index, ch, &fonts::STANDARD_LED_FONT)
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
            self.driver
                .write_raw_digit(device_index, row as u8, *value)?;
        }
        Ok(())
    }

    /// Draw a string of text on the LED matrix using the default font.
    /// Each character is displayed on one device in the daisy chain.
    /// If the string is longer than the number of devices, the extra characters are ignored.
    pub fn draw_text(&mut self, text: &str) -> Result<(), Error<SPI::Error>> {
        self.draw_text_with_font(text, &fonts::STANDARD_LED_FONT)
    }

    /// Draw a string of text on the LED matrix using a specified font.
    /// Each character is displayed on one device in the daisy chain.
    /// If the string is longer than the number of devices, the extra characters are ignored.
    pub fn draw_text_with_font(
        &mut self,
        text: &str,
        font: &LedFont,
    ) -> Result<(), Error<SPI::Error>> {
        let device_count = self.driver.device_count();

        let mut row_data = [[0u8; MAX_DISPLAYS]; 8];

        for (i, ch) in text.chars().take(device_count).enumerate() {
            let device_index = device_count - 1 - i;
            let bitmap = font.get_char(ch);
            for (row, &value) in bitmap.iter().enumerate() {
                row_data[row][device_index] = value;
            }
        }

        // Each digit_register targets the same row index (0 to 7) in every device.
        // Example: if digit_register = Digit3 and device_count = 2,
        // then ops will look like:
        //     ops = [
        //         (Digit3, row_data[3][0]), // device 1 (farthest), row 3
        //         (Digit3, row_data[3][1]), // device 0 (nearest), row 3
        //     ];
        for (row_index, digit_register) in Register::digits().enumerate() {
            let ops_row = row_data[row_index];
            let mut ops = [(Register::NoOp, 0); MAX_DISPLAYS];

            for (device_index, op) in ops.iter_mut().take(device_count).enumerate() {
                *op = (digit_register, ops_row[device_index]);
            }

            self.driver.write_all_registers(&ops[..device_count])?;
        }

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
        let mut scroller = ScrollingText::new(text, &fonts::STANDARD_LED_FONT, config);
        scroller.reset();

        let device_count = self.driver().device_count();

        loop {
            // Store the original offset
            let base_offset = scroller.current_offset;

            // Update each display device
            for device_index in 0..device_count {
                // Set offset for this specific device
                // Each device shows 8 pixels, so device N shows pixels at offset + (N * 8)
                scroller.current_offset = base_offset + (device_index as i32 * 8);

                let frame = scroller.get_frame()?; // Each device shows 8 pixels width
                self.write_buffer(device_index, &frame)?;

                // Advance offset for next device to create continuous scrolling
                // scroller.current_offset += 8;
            }

            // Restore the original offset and step to next position
            scroller.current_offset = base_offset;

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

    /// Flush the internal display buffer to the actual LED matrix hardware.
    ///
    /// This function goes row by row (0 to 7), and for each row, it builds an array of
    /// SPI operations (`ops`) to send to all devices in the daisy-chained display.
    /// It packs each row of pixels into a single byte for each device, then sends the data
    /// using the driver's `write_all_registers` method.
    ///
    /// ### Example logic (DEVICE_COUNT = 2, row = 0):
    /// Assume self.framebuffer contains pixel bits for 2 devices (128 total):
    ///
    /// Device 0, row 0 pixels: [1, 0, 1, 0, 1, 0, 1, 0]  => 0b10101010 => 0xAA
    /// Device 1, row 0 pixels: [1, 1, 1, 1, 0, 0, 0, 0]  => 0b11110000 => 0xF0
    ///
    /// Since SPI sends left to right, we must reverse the device order in the ops array:
    ///     ops[0] = (Digit0, 0xF0)  // Device 1
    ///     ops[1] = (Digit0, 0xAA)  // Device 0
    ///
    /// These are sent out in one SPI write for Digit0, and similarly repeated for Digit1 through Digit7.
    pub fn flush(&mut self) -> Result<(), Error<SPI::Error>> {
        for (row, digit_register) in Register::digits().enumerate() {
            let mut ops = [(Register::NoOp, 0); MAX_DISPLAYS];

            for device_index in 0..DEVICE_COUNT {
                let buffer_start = device_index * 64 + row * 8;
                let mut packed_byte = 0;
                for col in 0..8 {
                    let pixel_index = buffer_start + col;
                    if pixel_index < self.framebuffer.len() && self.framebuffer[pixel_index] != 0 {
                        // bit 7 is leftmost pixel (Col 0) on the display
                        packed_byte |= 1 << (7 - col);
                    }
                }

                // Fill ops array in reverse order for SPI chain
                let ops_index = DEVICE_COUNT - 1 - device_index;
                ops[ops_index] = (digit_register, packed_byte);
            }

            self.driver.write_all_registers(&ops[..DEVICE_COUNT])?;
        }
        Ok(())
    }

    /// Clear the internal framebuffer (sets all pixels to 0).
    pub fn clear_buffer(&mut self) {
        self.framebuffer.fill(0);
    }

    /// Clear screen by resetting buffer and flushing
    pub fn clear_screen(&mut self) -> Result<(), Error<SPI::Error>> {
        self.clear_buffer();
        self.flush()
    }
}

#[cfg(feature = "graphics")]
mod eg_imports {
    pub use embedded_graphics_core::Pixel;

    pub use embedded_graphics_core::pixelcolor::BinaryColor;
    pub use embedded_graphics_core::prelude::{DrawTarget, OriginDimensions, Size};
}

#[cfg(feature = "graphics")]
use eg_imports::*;
#[cfg(feature = "graphics")]
use embedded_graphics_core::geometry::Dimensions;

// Implementing embedded-graphics DrawTarget for LedMatrix
#[cfg(feature = "graphics")]
impl<SPI, const BUFFER_LENGTH: usize, const DEVICE_COUNT: usize> DrawTarget
    for LedMatrix<SPI, BUFFER_LENGTH, DEVICE_COUNT>
where
    SPI: SpiDevice,
{
    type Color = BinaryColor;
    type Error = core::convert::Infallible; // Or your specific error type

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();
        for Pixel(pos, color) in pixels.into_iter() {
            if bb.contains(pos) {
                let device = (pos.x as usize) / 8;
                let col = (pos.x as usize) % 8;
                let row = pos.y as usize;

                if device < DEVICE_COUNT && row < 8 && col < 8 {
                    let index = device * 64 + row * 8 + col;
                    if index < self.framebuffer.len() {
                        self.framebuffer[index] = color.is_on() as u8;
                    }
                }
            }
        }
        // Note: Does not call self.flush() automatically.
        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<SPI, const BUFFER_LENGTH: usize, const DEVICE_COUNT: usize> OriginDimensions
    for LedMatrix<SPI, BUFFER_LENGTH, DEVICE_COUNT>
{
    fn size(&self) -> Size {
        Size::new(DEVICE_COUNT as u32 * 8, 8)
    }
}
