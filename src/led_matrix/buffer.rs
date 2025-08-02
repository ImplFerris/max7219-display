//! Matrix buffer for LED matrix operations

use crate::error::MatrixError;

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
    pub fn set_pixel(&mut self, x: u8, y: u8, state: bool) -> Result<(), MatrixError> {
        if x >= 8 || y >= 8 {
            return Err(MatrixError::BufferError);
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
    pub fn get_pixel(&self, x: u8, y: u8) -> Result<bool, MatrixError> {
        if x >= 8 || y >= 8 {
            return Err(MatrixError::BufferError);
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
    pub fn set_row(&mut self, row: u8, data: u8) -> Result<(), MatrixError> {
        if row >= 8 {
            return Err(MatrixError::BufferError);
        }

        self.data[row as usize] = data;
        Ok(())
    }

    /// Get a row from the buffer
    pub fn get_row(&self, row: u8) -> Result<u8, MatrixError> {
        if row >= 8 {
            return Err(MatrixError::BufferError);
        }

        Ok(self.data[row as usize])
    }
}

impl Default for MatrixBuffer {
    fn default() -> Self {
        Self::new()
    }
}
