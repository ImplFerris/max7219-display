//! Predefined 8x8 symbols for LED matrix

use crate::led_matrix::buffer::MatrixBuffer;

/// Enum representing predefined 8x8 symbols.
///
/// These symbols can be displayed on an LED matrix.
/// To convert an `Symbol` into a displayable buffer, use `.to_buffer()`.
#[derive(Debug, Clone, Copy)]
pub enum Symbol {
    /// Heart shape
    Heart,
    /// Smiley face
    Smiley,
    /// Sad face
    SadFace,
    /// Upward pointing arrow
    ArrowUp,
    /// Downward pointing arrow
    ArrowDown,
    /// Left pointing arrow
    ArrowLeft,
    /// Right pointing arrow
    ArrowRight,
    /// Checkmark symbol
    Checkmark,
    /// X mark symbol
    XMark,
    /// Musical note symbol
    MusicNote,
    /// circle
    Circle,
}

impl Symbol {
    /// Convert the selected symbol into a `MatrixBuffer` pattern.
    ///
    /// This returns an 8x8 matrix buffer representing the selected symbol.
    #[rustfmt::skip]
    pub const fn to_buffer(&self) -> MatrixBuffer {
        match self {
            Symbol::Heart => MatrixBuffer::from_data([
                0b00000000,
                0b01100110,
                0b11111111,
                0b11111111,
                0b11111111,
                0b01111110,
                0b00111100,
                0b00011000,
            ]),
            Symbol::Smiley => MatrixBuffer::from_data([
                0b00111100,
                0b01000010,
                0b10100101,
                0b10000001,
                0b10100101,
                0b10011001,
                0b01000010,
                0b00111100,
            ]),
             Symbol::SadFace => MatrixBuffer::from_data([
                0b00111100, 
                0b01000010, 
                0b10100101, 
                0b10000001, 
                0b10011001, 
                0b10100101, 
                0b01000010, 
                0b00111100, 
            ]),
            Symbol::ArrowUp => MatrixBuffer::from_data([
                0b00000000, 
                0b00010000,
                0b00111000,
                0b01111100,
                0b11111110,
                0b00010000,
                0b00010000,
                0b00010000,
            ]),
            Symbol::ArrowDown => MatrixBuffer::from_data([
                0b00010000, 
                0b00010000,
                0b00010000,
                0b11111110, 
                0b01111100,
                0b00111000,
                0b00010000,
                0b00000000
            ]),
            Symbol::ArrowLeft => MatrixBuffer::from_data([
                0b00001000,
                0b00011000,
                0b00111000,
                0b01111111,
                0b00111000,
                0b00011000,
                0b00001000,
                0b00000000,
            ]),
            Symbol::ArrowRight => MatrixBuffer::from_data([
                0b00001000,
                0b00001100,
                0b00001110,
                0b11111111,
                0b00001110,
                0b00001100,
                0b00001000,
                0b00000000,
            ]),
            Symbol::Checkmark => MatrixBuffer::from_data([
                0b00000001,
                0b00000010,
                0b00000100,
                0b10001000,
                0b01010000,
                0b00100000,
                0b00000000,
                0b00000000,
            ]),
            Symbol::XMark => MatrixBuffer::from_data([
                0b10000001,
                0b01000010,
                0b00100100,
                0b00011000,
                0b00011000,
                0b00100100,
                0b01000010,
                0b10000001,
            ]),
            Symbol::MusicNote => MatrixBuffer::from_data([
                0b00011100,
                0b00010100,
                0b00011100,
                0b00010100,
                0b11010100,
                0b11110100,
                0b01110000,
                0b00100000,
            ]),
            Symbol::Circle => MatrixBuffer::from_data([
                0b00111100,
                0b01111110,
                0b11111111,
                0b11111111,
                0b11111111,
                0b11111111,
                0b01111110,
                0b00111100,
            ]),
        }
    }
}
