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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_font_known_chars() {
        // Test a selection of characters from the standard font
        assert_eq!(STANDARD_FONT.get_char('0'), 0b01111110);
        assert_eq!(STANDARD_FONT.get_char('1'), 0b00110000);
        assert_eq!(STANDARD_FONT.get_char('2'), 0b01101101);
        assert_eq!(STANDARD_FONT.get_char('A'), 0b01110111);
        assert_eq!(STANDARD_FONT.get_char('F'), 0b01000111);
        assert_eq!(STANDARD_FONT.get_char('-'), 0b00000001);
        assert_eq!(STANDARD_FONT.get_char(' '), 0b00000000);
    }

    #[test]
    fn test_standard_font_unknown_char() {
        // Test that unknown characters return 0 (blank)
        assert_eq!(STANDARD_FONT.get_char('z'), 0b00000000);
        assert_eq!(STANDARD_FONT.get_char('€'), 0b00000000);
        assert_eq!(STANDARD_FONT.get_char('@'), 0b00000000);
    }

    #[test]
    fn test_custom_font() {
        // Create a custom font for testing
        const CUSTOM_FONT_DATA: &[(char, u8)] = &[('X', 0b10101010), ('Y', 0b01010101)];

        let custom_font = Font::new(CUSTOM_FONT_DATA);

        // Test known characters
        assert_eq!(custom_font.get_char('X'), 0b10101010);
        assert_eq!(custom_font.get_char('Y'), 0b01010101);

        // Test unknown character
        assert_eq!(custom_font.get_char('Z'), 0b00000000);
    }

    #[test]
    fn test_font_new() {
        // Test that Font::new creates a font with the correct character map
        const TEST_DATA: &[(char, u8)] = &[('A', 0xFF), ('B', 0x00)];
        let font = Font::new(TEST_DATA);

        // Verify the font works correctly
        assert_eq!(font.get_char('A'), 0xFF);
        assert_eq!(font.get_char('B'), 0x00);
        assert_eq!(font.get_char('C'), 0x00); // Unknown char
    }
}
