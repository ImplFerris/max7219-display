//! Font definitions for 7-segment displays

/// 7-segment font mapping
pub struct Font {
    char_map: &'static [(char, u8)],
}

impl Font {
    /// Create a new font from character mappings
    pub const fn new(char_map: &'static [(char, u8)]) -> Self {
        Self { char_map }
    }

    /// Get segment pattern for a character
    pub fn get_char(&self, ch: char) -> u8 {
        for &(font_char, segments) in self.char_map {
            if font_char == ch {
                return segments;
            }
        }
        0x00 // Return blank for unknown characters
    }
}

/// Standard 7-segment font
/// Bit mapping: DP G F E D C B A
///
/// ```text
///   A
/// F   B
///   G
/// E   C
///   D    DP
/// ```
pub const STANDARD_FONT: Font = Font::new(&[
    ('0', 0b01111110), // 0
    ('1', 0b00110000), // 1
    ('2', 0b01101101), // 2
    ('3', 0b01111001), // 3
    ('4', 0b00110011), // 4
    ('5', 0b01011011), // 5
    ('6', 0b01011111), // 6
    ('7', 0b01110000), // 7
    ('8', 0b01111111), // 8
    ('9', 0b01111011), // 9
    ('A', 0b01110111), // A
    ('B', 0b00011111), // b
    ('C', 0b01001110), // C
    ('D', 0b00111101), // d
    ('E', 0b01001111), // E
    ('F', 0b01000111), // F
    ('H', 0b00110111), // H
    ('L', 0b00001110), // L
    ('P', 0b01100111), // P
    ('U', 0b00111110), // U
    ('-', 0b00000001), // -
    (' ', 0b00000000), // Space
]);
