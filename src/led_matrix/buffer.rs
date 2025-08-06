//! Matrix buffer for LED matrix operations

use crate::{Error, Result};

/// 8x8 matrix buffer for LED matrix displays
#[derive(Debug, Clone)]
pub struct MatrixBuffer {
    data: [u8; 8],
}

impl MatrixBuffer {
    /// Create a new empty buffer
    pub const fn new() -> Self {
        Self { data: [0; 8] }
    }

    /// Create a buffer from raw data
    pub const fn from_data(data: [u8; 8]) -> Self {
        Self { data }
    }

    /// Get reference to buffer data
    pub fn data(&self) -> &[u8; 8] {
        &self.data
    }

    /// Get mutable reference to buffer data
    pub fn data_mut(&mut self) -> &mut [u8; 8] {
        &mut self.data
    }

    /// Set a pixel in the buffer
    pub fn set_pixel(&mut self, x: u8, y: u8, state: bool) -> Result<()> {
        if x >= 8 || y >= 8 {
            return Err(Error::BufferError);
        }

        let bit_mask = 1 << x;
        if state {
            self.data[y as usize] |= bit_mask;
        } else {
            self.data[y as usize] &= !bit_mask;
        }

        Ok(())
    }

    /// Get pixel state from buffer
    pub fn get_pixel(&self, x: u8, y: u8) -> Result<bool> {
        if x >= 8 || y >= 8 {
            return Err(Error::BufferError);
        }

        let bit_mask = 1 << x;
        Ok((self.data[y as usize] & bit_mask) != 0)
    }

    /// Clear the entire buffer
    pub fn clear(&mut self) {
        self.data = [0; 8];
    }

    /// Fill the entire buffer
    pub fn fill(&mut self) {
        self.data = [0xFF; 8];
    }

    /// Set a row in the buffer
    pub fn set_row(&mut self, row: u8, data: u8) -> Result<()> {
        if row >= 8 {
            return Err(Error::BufferError);
        }

        self.data[row as usize] = data;
        Ok(())
    }

    /// Get a row from the buffer
    pub fn get_row(&self, row: u8) -> Result<u8> {
        if row >= 8 {
            return Err(Error::BufferError);
        }

        Ok(self.data[row as usize])
    }
}

impl Default for MatrixBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = MatrixBuffer::new();
        assert_eq!(buffer.data(), &[0; 8]);
    }

    #[test]
    fn test_default_buffer() {
        let buffer: MatrixBuffer = Default::default();
        assert_eq!(buffer.data(), &[0; 8]);
    }

    #[test]
    fn test_from_data() {
        let data = [0xFF, 0x00, 0xAA, 0x55, 0xF0, 0x0F, 0xCC, 0x33];
        let buffer = MatrixBuffer::from_data(data);
        assert_eq!(buffer.data(), &data);
    }

    #[test]
    fn test_clear() {
        let mut buffer = MatrixBuffer::from_data([0xFF; 8]);
        buffer.clear();
        assert_eq!(buffer.data(), &[0; 8]);
    }

    #[test]
    fn test_fill() {
        let mut buffer = MatrixBuffer::new();
        buffer.fill();
        assert_eq!(buffer.data(), &[0xFF; 8]);
    }

    #[test]
    fn test_set_pixel_valid() {
        let mut buffer = MatrixBuffer::new();

        // Set some pixels
        assert!(buffer.set_pixel(0, 0, true).is_ok());
        assert!(buffer.set_pixel(7, 7, true).is_ok());
        assert!(buffer.set_pixel(3, 4, true).is_ok());

        // Check pixel states
        assert!(buffer.get_pixel(0, 0).unwrap());
        assert!(buffer.get_pixel(7, 7).unwrap());
        assert!(buffer.get_pixel(3, 4).unwrap());
        assert!(!buffer.get_pixel(1, 1).unwrap());
    }

    #[test]
    fn test_set_pixel_invalid() {
        let mut buffer = MatrixBuffer::new();

        // Test invalid coordinates
        assert_eq!(buffer.set_pixel(8, 0, true), Err(Error::BufferError));
        assert_eq!(buffer.set_pixel(0, 8, true), Err(Error::BufferError));
        assert_eq!(buffer.set_pixel(255, 0, true), Err(Error::BufferError));
        assert_eq!(buffer.set_pixel(0, 255, true), Err(Error::BufferError));
    }

    #[test]
    fn test_get_pixel_invalid() {
        let buffer = MatrixBuffer::new();

        // Test invalid coordinates
        assert_eq!(buffer.get_pixel(8, 0), Err(Error::BufferError));
        assert_eq!(buffer.get_pixel(0, 8), Err(Error::BufferError));
        assert_eq!(buffer.get_pixel(255, 0), Err(Error::BufferError));
        assert_eq!(buffer.get_pixel(0, 255), Err(Error::BufferError));
    }

    #[test]
    fn test_pixel_manipulation() {
        let mut buffer = MatrixBuffer::new();

        // Set a pixel
        buffer.set_pixel(2, 3, true).unwrap();
        assert!(buffer.get_pixel(2, 3).unwrap());

        // Clear the same pixel
        buffer.set_pixel(2, 3, false).unwrap();
        assert!(!buffer.get_pixel(2, 3).unwrap());

        // Test bit manipulation doesn't affect other bits
        buffer.set_pixel(0, 0, true).unwrap();
        buffer.set_pixel(7, 0, true).unwrap();
        assert!(buffer.get_pixel(0, 0).unwrap());
        assert!(buffer.get_pixel(7, 0).unwrap());
        assert!(!buffer.get_pixel(1, 0).unwrap());
        assert!(!buffer.get_pixel(6, 0).unwrap());
    }

    #[test]
    fn test_set_row_valid() {
        let mut buffer = MatrixBuffer::new();

        // Set rows with different values
        assert!(buffer.set_row(0, 0xFF).is_ok());
        assert!(buffer.set_row(7, 0x00).is_ok());
        assert!(buffer.set_row(3, 0xAA).is_ok());

        // Verify row values
        assert_eq!(buffer.get_row(0).unwrap(), 0xFF);
        assert_eq!(buffer.get_row(7).unwrap(), 0x00);
        assert_eq!(buffer.get_row(3).unwrap(), 0xAA);
    }

    #[test]
    fn test_set_row_invalid() {
        let mut buffer = MatrixBuffer::new();

        // Test invalid row indices
        assert_eq!(buffer.set_row(8, 0xFF), Err(Error::BufferError));
        assert_eq!(buffer.set_row(255, 0xFF), Err(Error::BufferError));
    }

    #[test]
    fn test_get_row_invalid() {
        let buffer = MatrixBuffer::new();

        // Test invalid row indices
        assert_eq!(buffer.get_row(8), Err(Error::BufferError));
        assert_eq!(buffer.get_row(255), Err(Error::BufferError));
    }

    #[test]
    fn test_data_mut() {
        let mut buffer = MatrixBuffer::new();
        let data_mut = buffer.data_mut();

        // Modify the buffer through the mutable reference
        data_mut[0] = 0b10101010;
        data_mut[1] = 0b01010101;

        // Confirm that the internal data was updated
        assert_eq!(buffer.data()[0], 0b10101010);
        assert_eq!(buffer.data()[1], 0b01010101);
    }
}
