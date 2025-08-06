//! LED matrix display implementation

use embedded_hal::{delay::DelayNs, spi::SpiDevice};

use crate::{
    Error, MAX_DISPLAYS, Max7219, Register, Result,
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
    /// let mut matrix = SingleMatrix::from_spi(spi, 4).unwrap();
    /// ```
    pub fn from_spi(spi: SPI) -> Result<Self> {
        let mut driver = Max7219::new(spi).with_device_count(DEVICE_COUNT)?;
        driver.init()?;
        Ok(Self {
            driver,
            framebuffer: [0; BUFFER_LENGTH],
        })
    }

    /// Creates a new `LedMatrix` instance from an existing `Max7219` driver.
    ///
    /// This method is useful if you have already created and configured a `Max7219` driver manually.
    /// In most cases, it is recommended to use [`Self::from_spi`] instead, which creates the driver
    /// and matrix together in one step.
    ///
    ///
    ///
    /// # Arguments
    ///
    /// * `driver` - An initialized `Max7219` driver instance.
    ///
    /// # Error
    ///
    /// Returns `Err(Error::InvalidDeviceCount)` if the driver's device count
    /// does not match the generic `DEVICE_COUNT` parameter of this matrix type.
    ///
    /// # Warning
    ///
    /// This method is more error-prone than [`Self::from_spi`] because it is easy to configure a driver
    /// with one device count (e.g., `.with_device_count(4)`) and then call `from_driver` on a `LedMatrix`
    /// type instantiated with a different generic parameter (e.g., `LedMatrix<_, 1>`).
    /// This mismatch will result in an error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let driver = Max7219::new(spi).with_device_count(1).expect("device count 1 should not panic");
    /// let mut matrix = SingleMatrix::new(driver).unwrap();
    /// ```
    pub fn from_driver(driver: Max7219<SPI>) -> Result<Self> {
        if driver.device_count() != DEVICE_COUNT {
            return Err(Error::InvalidDeviceCount);
        }
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
    pub fn clear(&mut self, device_index: usize) -> Result<()> {
        self.driver.clear_display(device_index)
    }

    /// Clear all device
    pub fn clear_all(&mut self) -> Result<()> {
        self.driver.clear_all()
    }

    /// Write a complete buffer to a specific display
    pub fn write_buffer(&mut self, device_index: usize, buffer: &MatrixBuffer) -> Result<()> {
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
    pub fn draw_char(&mut self, device_index: usize, ch: char) -> Result<()> {
        self.draw_char_with_font(device_index, ch, &fonts::STANDARD_LED_FONT)
    }

    /// Draws a single 8x8 character on the specified display device using a provided font.
    ///
    /// This function is similar to [`Self::draw_char`], but allows overriding the font used for rendering.
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
    ) -> Result<()> {
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
    pub fn draw_text(&mut self, text: &str) -> Result<()> {
        self.draw_text_with_font(text, &fonts::STANDARD_LED_FONT)
    }

    /// Draw a string of text on the LED matrix using a specified font.
    /// Each character is displayed on one device in the daisy chain.
    /// If the string is longer than the number of devices, the extra characters are ignored.
    pub fn draw_text_with_font(&mut self, text: &str, font: &LedFont) -> Result<()> {
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
    ) -> Result<()> {
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
    pub fn scroll_text_default<D: DelayNs>(&mut self, delay: &mut D, text: &str) -> Result<()> {
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
    ///     ops\[0\] = (Digit0, 0xF0)  // Device 1
    ///     ops\[1\] = (Digit0, 0xAA)  // Device 0
    ///
    /// These are sent out in one SPI write for Digit0, and similarly repeated for Digit1 through Digit7.
    pub fn flush(&mut self) -> Result<()> {
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
    pub fn clear_screen(&mut self) -> Result<()> {
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
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> core::result::Result<(), Self::Error>
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

#[cfg(test)]
mod tests {
    use crate::Error;
    use crate::led_matrix::display::{Matrix4, SingleMatrix};
    use crate::led_matrix::fonts::STANDARD_LED_FONT;
    use crate::led_matrix::{LedMatrix, buffer::MatrixBuffer, fonts::LedFont};
    use crate::registers::Register;
    use crate::{Max7219, NUM_DIGITS};
    use embedded_hal_mock::eh1::{spi::Mock as SpiMock, spi::Transaction};

    fn write_reg(addr: u8, value: u8) -> Vec<Transaction<u8>> {
        vec![
            Transaction::transaction_start(),
            Transaction::write_vec(vec![addr, value]),
            Transaction::transaction_end(),
        ]
    }

    #[test]
    fn test_new() {
        let mut spi = SpiMock::new(&[]);
        let driver = Max7219::new(&mut spi);
        let matrix: LedMatrix<_, 64, 1> = LedMatrix::from_driver(driver).unwrap();
        assert_eq!(matrix.framebuffer, [0u8; 64]);
        spi.done();
    }

    #[test]
    fn test_from_spi() {
        let mut expected_transactions: Vec<Transaction<u8>> = vec![];

        // power_on
        expected_transactions.extend(write_reg(Register::Shutdown.addr(), 0x01));

        // test_all(false)
        expected_transactions.extend(write_reg(Register::DisplayTest.addr(), 0x00));

        // set_scan_limit_all(NUM_DIGITS)
        expected_transactions.extend(write_reg(Register::ScanLimit.addr(), NUM_DIGITS - 1));

        // set_decode_mode_all(NoDecode)
        expected_transactions.extend(write_reg(
            Register::DecodeMode.addr(),
            crate::registers::DecodeMode::NoDecode as u8,
        ));

        // clear_all() - 8 digits/rows
        for digit in Register::digits() {
            expected_transactions.extend(write_reg(digit.addr(), 0x00));
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let result: crate::Result<LedMatrix<_, 64, 1>> = LedMatrix::from_spi(&mut spi);

        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_from_spi_invalid_count() {
        let mut spi = SpiMock::new(&[]);
        let driver = Max7219::new(&mut spi);
        // not valid count
        let result = LedMatrix::<_, 1024, 256>::from_driver(driver);
        assert!(matches!(result, Err(Error::InvalidDeviceCount)));

        // Mismatched device count
        let driver = Max7219::new(&mut spi);
        let result = Matrix4::from_driver(driver);
        assert!(matches!(result, Err(Error::InvalidDeviceCount)));

        spi.done();
    }

    #[test]
    fn test_clear() {
        let mut expected_transactions = vec![];
        for digit_register in Register::digits() {
            expected_transactions.extend(write_reg(digit_register.addr(), 0x00));
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).expect("valid initialization");

        let result = matrix.clear(0);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_clear_all() {
        // Test clearing all devices (but checking with 1 device for SingleMatrix)
        let mut expected_transactions = vec![];
        let device_count = 4;

        for digit_register in Register::digits() {
            expected_transactions.push(Transaction::transaction_start());
            let batched_data = (0..device_count)
                .flat_map(|_| vec![digit_register.addr(), 0x00])
                .collect();
            expected_transactions.push(Transaction::write_vec(batched_data));
            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi).with_device_count(4).unwrap();
        let mut matrix = Matrix4::from_driver(driver).expect("valid initialization");

        let result = matrix.clear_all();
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_write_buffer() {
        let device_index = 0;
        let mut buffer = MatrixBuffer::new();
        // Fill buffer with some test data
        for i in 0..8 {
            buffer.set_row(i, 0b10101010 << (i % 2)).unwrap();
        }
        let expected_data = buffer.data();

        let mut expected_transactions = Vec::new();
        for (row, &data) in expected_data.iter().enumerate() {
            expected_transactions.push(Transaction::transaction_start());
            expected_transactions.push(Transaction::write_vec(vec![
                Register::try_digit(row as u8).unwrap().addr(),
                data,
            ]));
            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).expect("valid initialization");

        let result = matrix.write_buffer(device_index, &buffer);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_write_buffer_invalid_index() {
        let mut spi = SpiMock::new(&[]); // No SPI calls expected
        let driver = Max7219::new(&mut spi).with_device_count(1).unwrap();
        let mut matrix = SingleMatrix::from_driver(driver).expect("valid initialization");
        let buffer = MatrixBuffer::new();

        let result = matrix.write_buffer(1, &buffer); // Index 1 is invalid for device_count=1
        // This error comes from Max7219::write_raw_digit via write_device_register
        assert_eq!(result, Err(Error::InvalidDeviceIndex));
        spi.done();
    }

    #[test]
    fn test_draw_char() {
        let device_index = 0;
        let ch = 'A';
        let expected_bitmap = STANDARD_LED_FONT.get_char(ch);

        let mut expected_transactions = Vec::new();
        for (row, &data) in expected_bitmap.iter().enumerate() {
            expected_transactions.push(Transaction::transaction_start());
            expected_transactions.push(Transaction::write_vec(vec![
                Register::try_digit(row as u8).unwrap().addr(),
                data,
            ]));
            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).expect("valid initialization");

        let result = matrix.draw_char(device_index, ch);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_draw_char_with_font() {
        let device_index = 0;
        let ch = 'Z';
        let test_font = STANDARD_LED_FONT;
        let expected_bitmap = test_font.get_char(ch);

        let mut expected_transactions = Vec::new();
        for (row, &data) in expected_bitmap.iter().enumerate() {
            expected_transactions.push(Transaction::transaction_start());
            expected_transactions.push(Transaction::write_vec(vec![
                Register::try_digit(row as u8).unwrap().addr(),
                data,
            ]));
            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).expect("valid initialization");

        matrix
            .draw_char_with_font(device_index, ch, &test_font)
            .expect("valid character");
        spi.done();
    }

    #[test]
    fn test_draw_char_invalid_index() {
        let mut spi = SpiMock::new(&[]); // No SPI calls expected
        let driver = Max7219::new(&mut spi).with_device_count(1).unwrap();
        let mut matrix = SingleMatrix::from_driver(driver).expect("valid initialization");

        let result = matrix.draw_char(1, 'A'); // Index 1 is invalid for device_count=1
        // This error comes from Max7219::write_raw_digit via write_device_register
        assert_eq!(result, Err(Error::InvalidDeviceIndex));
        spi.done();
    }

    #[test]
    fn test_draw_text_single_device() {
        let text = "H";
        let expected_ch = 'H';
        let expected_bitmap = STANDARD_LED_FONT.get_char(expected_ch);
        // For 1 device, text is written to device 0 (device_count - 1 - i = 0)
        // Data is sent row by row using write_all_registers

        let mut expected_transactions = Vec::new();
        for (row_index, &data) in expected_bitmap.iter().enumerate() {
            let digit_register = Register::try_digit(row_index as u8).unwrap();
            expected_transactions.push(Transaction::transaction_start());
            // For 1 device, ops[0] = (digit_register, data)
            expected_transactions.push(Transaction::write_vec(vec![digit_register.addr(), data]));
            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).expect("valid initialization");

        let result = matrix.draw_text(text);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_draw_text_multi_device() {
        let device_count = 4;

        let text = "Hi";
        let expected_bitmap_h = STANDARD_LED_FONT.get_char('H');
        let expected_bitmap_i = STANDARD_LED_FONT.get_char('i');
        // Text chars are taken in order: 'H', 'i'
        // They are written to devices in reverse order: 'H' -> device 1, 'i' -> device 0
        // (because device_index = device_count - 1 - i)

        let mut expected_transactions = Vec::new();
        // Row by row transmission
        for row_index in 0..8 {
            let digit_register = Register::try_digit(row_index).unwrap();
            expected_transactions.push(Transaction::transaction_start());
            // ops[0] = (digit_register, data_for_device_1) // Sent first on SPI
            // ops[1] = (digit_register, data_for_device_0) // Sent second, ends up in device 0
            // Remaining devices gets filled with 0
            let data_for_device_1 = expected_bitmap_h[row_index as usize];
            let data_for_device_0 = expected_bitmap_i[row_index as usize];
            expected_transactions.push(Transaction::write_vec(vec![
                digit_register.addr(),
                data_for_device_1, // Data for device 1
                digit_register.addr(),
                data_for_device_0, // Data for device 0
                digit_register.addr(),
                0x00,
                digit_register.addr(),
                0x00,
            ]));
            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi)
            .with_device_count(device_count)
            .unwrap();
        let mut matrix = Matrix4::from_driver(driver).unwrap();

        let result = matrix.draw_text(text);
        assert!(result.is_ok());
        spi.done();
    }

    #[test]
    fn test_draw_text_with_font() {
        let text = "!";
        // Create a simple test font
        pub const TEST_FONT: &[([u8; 8], char)] = &[([0b10101010; 8], '!')];
        let test_font = LedFont::new(TEST_FONT);
        let expected_bitmap = [0b10101010u8; 8];
        let device_count = 2;

        let mut expected_transactions = Vec::new();

        // For each row (Digit0 to Digit7)
        for (row_index, &data) in expected_bitmap.iter().enumerate() {
            let digit_register = Register::try_digit(row_index as u8).unwrap();
            expected_transactions.push(Transaction::transaction_start());

            // First device gets the font data, second device gets zero
            expected_transactions.push(Transaction::write_vec(vec![
                digit_register.addr(),
                data, // Device 0
                digit_register.addr(),
                0x00, // Device 1 (no text)
            ]));

            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi)
            .with_device_count(device_count)
            .unwrap();
        let mut matrix: LedMatrix<_, 128, 2> = LedMatrix::from_driver(driver).unwrap();

        let result = matrix.draw_text_with_font(text, &test_font);
        assert!(result.is_ok());

        spi.done();
    }

    #[test]
    fn test_clear_buffer() {
        let mut spi = SpiMock::new(&[]); // No SPI interaction
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).unwrap();

        // Modify the buffer
        matrix.framebuffer[0] = 1;
        matrix.framebuffer[10] = 1;
        matrix.framebuffer[63] = 1;
        assert_ne!(matrix.framebuffer, [0u8; 64]);

        matrix.clear_buffer();

        assert_eq!(matrix.framebuffer, [0u8; 64]);
        spi.done();
    }

    #[test]
    fn test_clear_screen() {
        // All digits 0..7 will be written with 0x00 for a single device
        let mut expected_transactions = Vec::new();
        for row in 0..8 {
            let digit_register = Register::try_digit(row).unwrap();
            expected_transactions.push(Transaction::transaction_start());
            expected_transactions.push(Transaction::write_vec(vec![digit_register.addr(), 0x00]));
            expected_transactions.push(Transaction::transaction_end());
        }

        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).unwrap();

        // Modify the buffer
        matrix.framebuffer[5] = 1;
        matrix.framebuffer[15] = 1;

        assert_ne!(matrix.framebuffer, [0u8; 64]);

        let result = matrix.clear_screen();
        assert!(result.is_ok());
        assert_eq!(matrix.framebuffer, [0u8; 64]);
        spi.done();
    }

    #[test]
    fn test_flush_single_device() {
        // We expect the flush to send 8 SPI transactions, one for each row (DIGIT0 to DIGIT7)
        // Only rows 0 and 7 have pixel data: 0b10101010 (columns 0,2,4,6 lit)
        // All other rows should be cleared (0b00000000)

        let mut expected_transactions = Vec::new();
        for (row, digit_register) in Register::digits().enumerate() {
            // For rows 0 and 7, the framebuffer will result in this pattern:
            // Columns 0, 2, 4, 6 are ON => bits 7, 5, 3, 1 set => 0b10101010
            let expected_byte = if row == 0 || row == 7 {
                0b10101010
            } else {
                0b00000000
            };

            // Each transaction sends [register, data] for that row
            expected_transactions.push(Transaction::transaction_start());
            expected_transactions.push(Transaction::write_vec(vec![
                digit_register.addr(),
                expected_byte,
            ]));
            expected_transactions.push(Transaction::transaction_end());
        }

        // Create the SPI mock with the expected sequence of writes
        let mut spi = SpiMock::new(&expected_transactions);
        let driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(driver).unwrap();

        // Set framebuffer values to light up alternating columns in row 0 and row 7
        // Row 0 corresponds to framebuffer indices 0 to 7
        matrix.framebuffer[0] = 1; // Column 0
        matrix.framebuffer[2] = 1; // Column 2
        matrix.framebuffer[4] = 1; // Column 4
        matrix.framebuffer[6] = 1; // Column 6

        // Each device's framebuffer is a flat array of 64 bytes: 8 rows * 8 columns
        // The layout is row-major: [row0[0..7], row1[0..7], ..., row7[0..7]]
        //
        // For a single device:
        //   framebuffer[ 0.. 7] => row 0
        //   framebuffer[ 8..15] => row 1
        //   framebuffer[16..23] => row 2
        //   framebuffer[24..31] => row 3
        //   framebuffer[32..39] => row 4
        //   framebuffer[40..47] => row 5
        //   framebuffer[48..55] => row 6
        //   framebuffer[56..63] => row 7 (last row)
        //
        // So to update row 7, we write to indices 56 to 63.
        matrix.framebuffer[56] = 1; // Column 0
        matrix.framebuffer[58] = 1; // Column 2
        matrix.framebuffer[60] = 1; // Column 4
        matrix.framebuffer[62] = 1; // Column 6

        // Call flush, which will convert framebuffer rows into bytes and send via SPI
        let result = matrix.flush();
        assert!(result.is_ok());

        spi.done();
    }

    #[test]
    fn test_driver_mut_access() {
        let expected_transactions = [
            Transaction::transaction_start(),
            Transaction::write_vec(vec![Register::Shutdown.addr(), 0x01]),
            Transaction::transaction_end(),
        ];
        let mut spi = SpiMock::new(&expected_transactions);
        let original_driver = Max7219::new(&mut spi);
        let mut matrix = SingleMatrix::from_driver(original_driver).unwrap();

        let driver = matrix.driver();

        driver.power_on().expect("Power on should succeed");
        spi.done();
    }
}
