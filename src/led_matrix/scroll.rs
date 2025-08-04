//! Scroll Text Renderer Module
//!
//! This module provides a configurable text scroller for 8x8 LED matrix

use crate::{
    error::MatrixError,
    led_matrix::{buffer::MatrixBuffer, fonts::LedFont},
};

/// Configuration for scrolling text behavior
#[derive(Clone, Copy)]
pub struct ScrollConfig {
    /// Delay between scroll steps in nanoseconds
    pub step_delay_ns: u32,
    /// Number of pixels to scroll per step (usually 1 for smooth scrolling)
    pub pixels_per_step: u8,
    /// Whether to loop the text continuously
    pub loop_text: bool,
    /// Padding between text repetitions when looping (in pixels)
    pub loop_padding: u8,
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            step_delay_ns: 100_000_000, // 100ms
            pixels_per_step: 1,
            loop_text: true,
            loop_padding: 16, // 2 character widths
        }
    }
}

/// Scrolling text renderer for LED matrix displays
pub struct ScrollingText<'a> {
    text: &'a str,
    font: &'a LedFont,
    config: ScrollConfig,
    text_width: usize,
    pub(crate) current_offset: i32,
}

impl<'a> ScrollingText<'a> {
    /// Create a new scrolling text instance
    pub fn new(text: &'a str, font: &'a LedFont, config: ScrollConfig) -> Self {
        let mut scroller = Self {
            text,
            font,
            config,
            text_width: 0,
            current_offset: 0,
        };
        scroller.calculate_text_width();
        scroller
    }

    /// Create with default configuration
    pub fn new_default(text: &'a str, font: &'a LedFont) -> Self {
        Self::new(text, font, ScrollConfig::default())
    }

    /// Calculate the width
    fn calculate_text_width(&mut self) {
        self.text_width = self.text.chars().count() * 8;

        // Add loop padding if configured
        if self.config.loop_text {
            self.text_width += self.config.loop_padding as usize;
        }
    }

    /// Get the current 8x8 frame data based on the scroll offset.
    /// This returns what should be displayed on the LED matrix at the current scroll position.
    pub fn get_frame(&self) -> Result<MatrixBuffer, MatrixError> {
        let mut buffer = MatrixBuffer::new();

        for row in 0..8 {
            let mut row_data = 0u8;
            for col in 0..8 {
                if self.pixel_on(col, row) {
                    row_data |= 1 << (7 - col);
                }
            }
            buffer.set_row(row as u8, row_data)?;
        }

        Ok(buffer)
    }
    /// Return true if the pixel at (source_col, row) should be on
    fn pixel_on(&self, source_col: usize, row: usize) -> bool {
        // Calculate the actual column position considering the offset
        let actual_col = self.current_offset as isize + source_col as isize;

        // If the actual column is negative, no pixel should be on
        if actual_col < 0 {
            return false;
        }

        let col = actual_col as usize;

        // If outside text width and not looping, no pixel
        if col >= self.text_width && !self.config.loop_text {
            return false;
        }

        // Wrap around if looping
        let final_col = if self.config.loop_text && col >= self.text_width {
            col % self.text_width
        } else {
            col
        };

        // Only actual text columns (exclude padding)
        let text_pixels = self.text.chars().count() * 8;
        if final_col >= text_pixels {
            return false;
        }

        let char_index = final_col / 8;
        let bit_index = final_col % 8;

        // Safe since char_index < char count
        let ch = self.text.chars().nth(char_index).unwrap_or('?');
        let bitmap = self.font.get_char(ch);
        let row_data = bitmap[row];

        // Check bit (left to right)
        (row_data >> (7 - bit_index)) & 1 != 0
    }

    /// Advance the scroll position by the configured step size
    pub fn step(&mut self) -> bool {
        self.current_offset += self.config.pixels_per_step as i32;

        if self.config.loop_text {
            // Reset when we've scrolled past the text width
            if self.current_offset >= self.text_width as i32 {
                self.current_offset = 0;
            }
            true // Always continue when looping
        } else {
            // Stop when text has completely scrolled off screen
            self.current_offset < (self.text_width as i32 + 8)
        }
    }

    /// Reset scroll position to the beginning
    pub fn reset(&mut self) {
        self.current_offset = -(8i32); // Start with text off-screen to the right
    }

    /// Get current scroll offset
    pub fn offset(&self) -> i32 {
        self.current_offset
    }
}
