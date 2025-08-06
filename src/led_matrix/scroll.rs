//! Scroll Text Renderer Module
//!
//! This module provides a configurable text scroller for 8x8 LED matrix

use crate::{
    Result,
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
    pub fn get_frame(&self) -> Result<MatrixBuffer> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Create a minimal font for testing
    #[rustfmt::skip]
    const TEST_FONT_DATA: &[([u8; 8], char)] = &[
        (
            [
                0b00111100, 
                0b01100110, 
                0b01101110, 
                0b01110110, 
                0b01100110, 
                0b01100110, 
                0b00111100,
                0b00000000,
            ],
            '0',
        ),
        (
            [
                0b00011000, 
                0b00111000, 
                0b00011000, 
                0b00011000, 
                0b00011000, 
                0b00011000, 
                0b01111110,
                0b00000000,
            ],
            '1',
        ),
        (
            [
                0b00000000, 
                0b00000000, 
                0b00000000, 
                0b00000000, 
                0b00000000, 
                0b00000000, 
                0b00000000,
                0b00000000,
            ],
            ' ',
        ),
    ];

    const TEST_FONT: LedFont = LedFont::new(TEST_FONT_DATA);
    #[test]
    fn test_scroll_config_default() {
        let config = ScrollConfig::default();
        assert_eq!(config.step_delay_ns, 100_000_000);
        assert_eq!(config.pixels_per_step, 1);
        assert!(config.loop_text);
        assert_eq!(config.loop_padding, 16);
    }

    #[test]
    fn test_scrolling_text_new() {
        let text = "01";
        let config = ScrollConfig::default();
        let scroller = ScrollingText::new(text, &TEST_FONT, config);

        assert_eq!(scroller.text, text);
        assert_eq!(scroller.text_width, 32);
        assert_eq!(scroller.current_offset, 0);
    }

    #[test]
    fn test_calculate_text_width() {
        let text = "01";
        let config = ScrollConfig {
            loop_text: true,
            loop_padding: 8,
            ..Default::default()
        };
        let scroller = ScrollingText::new(text, &TEST_FONT, config);
        assert_eq!(scroller.text_width, 24);
    }

    #[test]
    fn test_reset() {
        let mut scroller = ScrollingText::new_default("01", &TEST_FONT);
        scroller.current_offset = 10;
        scroller.reset();
        assert_eq!(scroller.current_offset, -8);
    }

    #[test]
    fn test_offset() {
        let mut scroller = ScrollingText::new_default("01", &TEST_FONT);
        assert_eq!(scroller.offset(), 0);
        scroller.current_offset = 5;
        assert_eq!(scroller.offset(), 5);
    }

    #[test]
    fn test_step_looping() {
        let mut scroller = ScrollingText::new_default("01", &TEST_FONT);
        scroller.current_offset = 30;

        let should_continue = scroller.step();
        assert_eq!(scroller.current_offset, 31);
        assert!(should_continue);

        let should_continue = scroller.step();
        assert_eq!(scroller.current_offset, 0);
        assert!(should_continue);
    }

    #[test]
    fn test_step_non_looping() {
        let config = ScrollConfig {
            loop_text: false,
            ..Default::default()
        };
        let mut scroller = ScrollingText::new("01", &TEST_FONT, config);

        // Text width is 24 (2 chars * 8 + 8 padding), so scrolling stops when offset >= 24
        scroller.current_offset = 22;
        assert!(scroller.step());
        assert_eq!(scroller.current_offset, 23);

        assert!(!scroller.step()); // Now completely off screen
        assert_eq!(scroller.current_offset, 24);

        // Test that it stays false
        assert!(!scroller.step()); // Still off screen
        assert_eq!(scroller.current_offset, 25);
    }

    #[test]
    fn test_pixel_on_basic() {
        let scroller = ScrollingText::new_default("0", &TEST_FONT);

        assert!(!scroller.pixel_on(0, 0));
        assert!(!scroller.pixel_on(1, 0));
        assert!(scroller.pixel_on(2, 0));
        assert!(scroller.pixel_on(3, 0));
        assert!(scroller.pixel_on(4, 0));
        assert!(scroller.pixel_on(5, 0));
        assert!(!scroller.pixel_on(6, 0));
        assert!(!scroller.pixel_on(7, 0));
    }

    #[test]
    fn test_pixel_on_negative_offset() {
        let mut scroller = ScrollingText::new_default("0", &TEST_FONT);
        scroller.current_offset = -4;

        // Columns 0-3 map to actual_cols -4 to -1, which are < 0, so pixel_on returns false
        // i.e off the screen
        assert!(!scroller.pixel_on(0, 0));
        assert!(!scroller.pixel_on(1, 0));
        assert!(!scroller.pixel_on(2, 0));
        assert!(!scroller.pixel_on(3, 0));

        // Column 4 maps to actual_col 0.
        // The bitmap for '0' (from TEST_FONT) row 0 is 0b00111100.
        // Bit 0 (rightmost shift) is 0.
        assert!(!scroller.pixel_on(4, 0)); // This should be false based on the actual font data

        // Column 5 maps to actual_col 1.
        // Bit 1 of 0b00111100 is 0.
        assert!(!scroller.pixel_on(5, 0)); // This should also be false based on the actual font data

        // Column 6 maps to actual_col 2. This is the third pixel of '0'.
        // Bit 2 of 0b00111100 is 1.
        assert!(scroller.pixel_on(6, 0)); // This should be true based on the actual font data
    }

    #[test]
    fn test_pixel_on_multiple_characters() {
        let scroller = ScrollingText::new_default("01", &TEST_FONT);

        assert!(!scroller.pixel_on(0, 0));
        assert!(!scroller.pixel_on(1, 0));
        assert!(scroller.pixel_on(2, 0));

        assert!(!scroller.pixel_on(8, 0));
        assert!(!scroller.pixel_on(9, 0));
        assert!(!scroller.pixel_on(10, 0));
        assert!(scroller.pixel_on(11, 0));
        assert!(scroller.pixel_on(12, 0));
    }

    #[test]
    fn test_pixel_on_looping() {
        let config = ScrollConfig {
            loop_text: true,
            loop_padding: 0,
            ..Default::default()
        };
        let mut scroller = ScrollingText::new("01", &TEST_FONT, config);

        scroller.current_offset = scroller.text_width as i32;

        assert!(!scroller.pixel_on(0, 0));
        assert!(!scroller.pixel_on(1, 0));
        assert!(scroller.pixel_on(2, 0));
    }

    #[test]
    fn test_pixel_on_padding() {
        let scroller = ScrollingText::new_default("0", &TEST_FONT);

        for col in 8..24 {
            assert!(
                !scroller.pixel_on(col, 0),
                "Pixel {col} should be off in padding",
            );
        }
    }

    #[test]
    fn test_get_frame() {
        let scroller = ScrollingText::new_default("0", &TEST_FONT);
        let frame = scroller.get_frame().expect("Should get frame successfully");

        // Expected bitmap for character '0' from TEST_FONT_DATA:
        // Row 0: 0b00111100
        // Row 1: 0b01100110
        // Row 2: 0b01101110
        // Row 3: 0b01110110
        // Row 4: 0b01100110
        // Row 5: 0b01100110
        // Row 6: 0b00111100
        // Row 7: 0b00000000
        //
        // get_frame processes pixels column by column (source_col 0-7) and builds
        // each row's byte by setting bits from left to right (bit 7 to bit 0).
        // So for row 0 with pixels [0,0,1,1,1,1,0,0], the byte becomes:
        // Bit 7 (col 0): 0
        // Bit 6 (col 1): 0
        // Bit 5 (col 2): 1
        // Bit 4 (col 3): 1
        // Bit 3 (col 4): 1
        // Bit 2 (col 5): 1
        // Bit 1 (col 6): 0
        // Bit 0 (col 7): 0
        // Result: 0b00111100
        //
        // This means the frame row data should match the character bitmap data exactly.

        let expected_rows = [
            0b00111100, // Row 0
            0b01100110, // Row 1
            0b01101110, // Row 2
            0b01110110, // Row 3
            0b01100110, // Row 4
            0b01100110, // Row 5
            0b00111100, // Row 6
            0b00000000, // Row 7
        ];

        for (row_index, &expected_row) in expected_rows.iter().enumerate() {
            let actual_row = frame
                .get_row(row_index as u8)
                .expect("Should get row successfully");
            assert_eq!(actual_row, expected_row, "Row {row_index} mismatch");
        }
    }
}
