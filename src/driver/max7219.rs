//! Core MAX7219 driver implementation

use embedded_hal::spi::SpiDevice;

use crate::{
    MAX_DISPLAYS, NUM_DIGITS,
    error::Error,
    registers::{DecodeMode, Digit, Register},
};

/// Driver for the MAX7219 LED display controller.
/// Communicates over SPI using the embedded-hal `SpiDevice` trait.
pub struct Max7219<SPI> {
    spi: SPI,
    buffer: [u8; MAX_DISPLAYS * 2],
    device_count: usize,
}

impl<SPI> Max7219<SPI>
where
    SPI: SpiDevice,
{
    /// Creates a new MAX7219 driver instance with the given SPI interface.
    ///
    /// The SPI interface must use Mode 0, which means the clock is low when idle
    /// and data is read on the rising edge of the clock signal.
    ///
    /// Defaults to a single device (can be daisy-chained using `with_device_count`).
    ///
    /// The SPI frequency must be 10 MHz or less, as required by the MAX7219 datasheet.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            device_count: 1, // Default to 1, use with_device_count to increase count
            buffer: [0; MAX_DISPLAYS * 2],
        }
    }

    /// Returns the number of MAX7219 devices managed by this driver.
    ///
    /// This corresponds to the number of daisy-chained MAX7219 units
    /// initialized during driver setup.
    pub fn device_count(&self) -> usize {
        self.device_count
    }

    /// Sets the number of daisy-chained devices to control.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidDisplayCount` if `count > MAX_DISPLAYS`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let driver = Max7219::new(spi).with_device_count(4)?;
    /// ```
    pub fn with_device_count(mut self, count: usize) -> Result<Self, Error<SPI::Error>> {
        if count > MAX_DISPLAYS {
            return Err(Error::InvalidDisplayCount);
        }
        self.device_count = count;
        Ok(self)
    }

    /// Initializes all configured displays.
    pub fn init(&mut self) -> Result<(), Error<SPI::Error>> {
        self.power_on()?;

        for i in 0..self.device_count {
            self.test_display(i, false)?;
            self.set_scan_limit(i, NUM_DIGITS)?;
            self.set_decode_mode(i, DecodeMode::NoDecode)?;
            self.clear_display(i)?;
        }

        self.power_off()?;
        self.power_on()?;

        Ok(())
    }

    /// Writes a value to a specific register of a device in the daisy chain.
    ///
    /// Each MAX7219 device expects a 16-bit packet: 1 byte for the register address
    /// and 1 byte for the data. To update one device in a daisy-chained series,
    /// we prepare a full SPI buffer of `display_count * 2` bytes (2 bytes per display).
    ///
    /// This method writes the target register and data into the correct offset of
    /// the shared buffer corresponding to the selected device (`device_index`),
    /// and clears the rest of the buffer. Then the entire buffer is sent via SPI.
    ///
    /// The device at `device_index` will receive its register update, while other
    /// devices in the chain will receive no-ops (zeros).
    ///
    /// # Arguments
    ///
    /// * `device_index` - Index of the device in the chain (0 = furthest from MCU, N-1 = nearest to MCU).
    /// * `register` - The register to write to (e.g., `Register::Shutdown`, `Register::Digit0`, etc.).
    /// * `data` - The value to write to the register.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidDisplayIndex` if the index is out of range, or an SPI error
    /// if the transfer fails.
    pub(crate) fn write_device_register(
        &mut self,
        device_index: usize,
        register: Register,
        data: u8,
    ) -> Result<(), Error<SPI::Error>> {
        if device_index >= self.device_count {
            return Err(Error::InvalidDisplayIndex);
        }

        self.buffer = [0; MAX_DISPLAYS * 2];

        let offset = device_index * 2; // 2 bytes(16 bits packet) per display
        self.buffer[offset] = register as u8;
        self.buffer[offset + 1] = data;

        self.spi.write(&self.buffer[0..self.device_count * 2])?;

        Ok(())
    }

    // fn write_raw_register(&mut self, register: u8, data: u8) -> Result<(), SPI::Error> {
    //     self.spi.write(&[register, data])
    // }

    /// Powers on all displays by writing `0x01` to the Shutdown register.
    pub fn power_on(&mut self) -> Result<(), Error<SPI::Error>> {
        for device_index in 0..self.device_count {
            self.power_on_display(device_index)?;
        }
        Ok(())
    }

    /// Powers off all displays by writing `0x00` to the Shutdown register.
    pub fn power_off(&mut self) -> Result<(), Error<SPI::Error>> {
        for device_index in 0..self.device_count {
            self.power_off_display(device_index)?;
        }
        Ok(())
    }

    /// Powers on a single display by writing `0x01` to the Shutdown register.
    ///
    /// # Arguments
    ///
    /// * `device_index` - The index of the display to power on.
    pub fn power_on_display(&mut self, device_index: usize) -> Result<(), Error<SPI::Error>> {
        self.write_device_register(device_index, Register::Shutdown, 0x01)
    }

    /// Powers off a single display by writing `0x00` to the Shutdown register.
    ///
    /// # Arguments
    ///
    /// * `device_index` - The index of the display to power off.
    pub fn power_off_display(&mut self, device_index: usize) -> Result<(), Error<SPI::Error>> {
        self.write_device_register(device_index, Register::Shutdown, 0x00)
    }

    /// Enables or disables display test mode on a specific display.
    ///
    /// When enabled, all LEDs on that display are lit regardless of current display data.
    pub fn test_display(
        &mut self,
        device_index: usize,
        enable: bool,
    ) -> Result<(), Error<SPI::Error>> {
        let data = if enable { 0x01 } else { 0x00 };
        self.write_device_register(device_index, Register::DisplayTest, data)
    }

    /// Sets how many digits the MAX7219 should actively scan and display.
    ///
    /// This tells the chip how many digit outputs (DIG0 to DIG7) should be used.
    /// The input value must be between 1 and 8:
    /// - 1 means only digit 0 is used
    /// - 8 means all digits (0 to 7) are used
    ///
    /// Internally, the value written to the chip is `limit - 1`, because the chip expects values from 0 to 7.
    ///
    /// This applies to a specific device in the daisy chain, selected by `device_index`.
    ///
    /// # Errors
    /// Returns `Error::InvalidScanLimit` if the value is not in the range 1 to 8.
    pub fn set_scan_limit(
        &mut self,
        device_index: usize,
        limit: u8,
    ) -> Result<(), Error<SPI::Error>> {
        if !(1..=8).contains(&limit) {
            return Err(Error::InvalidScanLimit);
        }

        self.write_device_register(device_index, Register::ScanLimit, limit - 1)
    }

    /// Sets which digits use Code B decoding mode.
    ///
    /// This determines whether the MAX7219 automatically decodes numeric values
    /// (like 0–9, E, H, L, etc.) for specific digits, or expects manual segment control.
    ///
    /// Use [`DecodeMode`] variants like `NoDecode`, `Digit0`, `Digits0To3`, or `AllDigits`
    /// depending on how many digits you want to enable for automatic decoding.
    ///
    /// This applies to a specific device in the daisy chain, selected by `device_index`.
    pub fn set_decode_mode(
        &mut self,
        device_index: usize,
        mode: DecodeMode,
    ) -> Result<(), Error<SPI::Error>> {
        self.write_device_register(device_index, Register::DecodeMode, mode as u8)
    }

    /// Clears all digits by writing 0 to each digit register (DIG0 to DIG7).
    ///
    /// This turns off all segments on the display by sending 0x00 to each of the
    /// digit registers (Register::Digit0 to Register::Digit7).
    ///
    /// This applies to a specific device in the daisy chain, selected by `device_index`.
    pub fn clear_display(&mut self, device_index: usize) -> Result<(), Error<SPI::Error>> {
        for digit in Digit::iter() {
            let register = Register::from(digit);
            self.write_device_register(device_index, register, 0x00)?;
        }
        Ok(())
    }

    /// Clears all digits on all connected MAX7219 displays.
    pub fn clear_all(&mut self) -> Result<(), Error<SPI::Error>> {
        for device_index in 0..self.device_count {
            self.clear_display(device_index)?;
        }
        Ok(())
    }

    /// Writes a raw value to the specified digit register (DIG0 to DIG7).
    ///
    /// This function gives you low-level control over the display by sending a
    /// raw 8-bit pattern to the specified digit. Each bit in the `value` corresponds
    /// to an individual segment (on 7-segment displays) or LED (on an LED matrix).
    ///
    /// **A typical 7-segment** display has the following layout:
    ///
    /// ```txt
    ///     A
    ///    ---
    /// F |   | B
    ///   |   |
    ///    ---
    /// E |   | C
    ///   |   |
    ///    ---   . DP
    ///     D
    /// ```
    ///
    /// | Byte        | 7  | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
    /// |-------------|----|---|---|---|---|---|---|---|
    /// | **Segment** | DP | A | B | C | D | E | F | G |
    ///
    /// For example, to display the number `1`, use the byte `0b00110000`,
    /// which lights up segments B and C.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// display.write_raw_digit(0, Digit::D0, 0b00110000)?; // Shows '1'
    /// ```
    ///
    /// **On an LED matrix (8x8)**, each digit register maps to a row, and each
    /// bit in the value maps to a column (from left to right).
    ///
    /// > **Note:** Wiring and orientation can vary between modules. Some modules map rows top-to-bottom,
    /// > others bottom-to-top; some wire columns left-to-right, others right-to-left.
    /// > If the display appears mirrored or rotated, adjust your digit and bit mapping accordingly.
    ///
    /// Here is an example layout for the FC-16 module, where DIG0 corresponds to the top row (row 0),
    /// and bit 0 maps to the rightmost column (column 0). So a value like `0b10101010` written to DIG0
    /// would light up every alternate LED across the top row from left to right.
    ///
    /// ```txt
    /// DIG0 -> Row 0: value = 0b10101010
    ///
    /// Matrix:
    ///           Columns
    ///            7 6 5 4 3 2 1 0
    ///          +----------------
    ///      0   | 1 0 1 0 1 0 1 0
    ///      1   | ...
    ///      2   | ...
    /// Rows 3   | ...
    ///      4   | ...
    ///      5   | ...
    ///      6   | ...
    ///      7   | ...
    /// ```
    ///
    /// This applies to a specific device in the daisy chain, selected by `device_index`.
    ///
    /// # Arguments
    ///
    /// - `device_index`: Index of the display in the daisy chain (0 = furthest from MCU)
    /// - `digit`: Which digit register to write to (`Digit::D0` to `Digit::D7`)
    /// - `value`: The raw 8-bit data to send to the digit register
    pub fn write_raw_digit(
        &mut self,
        device_index: usize,
        digit: Digit,
        value: u8,
    ) -> Result<(), Error<SPI::Error>> {
        let register = Register::from(digit);
        self.write_device_register(device_index, register, value)
    }

    /// Sets the brightness intensity (0 to 15) for a specific device.
    ///
    /// # Arguments
    ///
    /// - `device_index`: Index of the display in the daisy chain (0 = furthest from MCU)
    /// - `intensity`: Brightness level from `0` to `15` (`0x00` to `0x0F`)
    pub fn set_intensity(
        &mut self,
        device_index: usize,
        intensity: u8,
    ) -> Result<(), Error<SPI::Error>> {
        if intensity > 0x0F {
            return Err(Error::InvalidIntensity);
        }
        self.write_device_register(device_index, Register::Intensity, intensity)
    }

    /// Set intensity for all displays
    pub fn set_intensity_all(&mut self, intensity: u8) -> Result<(), Error<SPI::Error>> {
        for device_index in 0..self.device_count {
            self.set_intensity(device_index, intensity)?;
        }
        Ok(())
    }
}
