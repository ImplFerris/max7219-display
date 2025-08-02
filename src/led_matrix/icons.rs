//! Predefined 8x8 icons for LED matrix

use crate::led_matrix::buffer::MatrixBuffer;

/// Enum representing predefined 8x8 icon patterns.
///
/// These icons can be displayed on an LED matrix.
/// To convert an `Icon` into a displayable buffer, use `.to_buffer()`.
#[derive(Debug, Clone, Copy)]
pub enum Icon {
    /// Heart shape
    Heart,
    /// Smiley face
    Smiley,
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

impl Icon {
    /// Convert the selected icon into a `MatrixBuffer` pattern.
    ///
    /// This returns an 8x8 matrix buffer representing the selected icon.
    #[rustfmt::skip]
    pub const fn to_buffer(&self) -> MatrixBuffer {
        match self {
            Icon::Heart => MatrixBuffer::from_data([
                0b00000000,
                0b01100110,
                0b11111111,
                0b11111111,
                0b11111111,
                0b01111110,
                0b00111100,
                0b00011000,
            ]),
            Icon::Smiley => MatrixBuffer::from_data([
                0b00111100,
                0b01000010,
                0b10100101,
                0b10000001,
                0b10100101,
                0b10011001,
                0b01000010,
                0b00111100,
            ]),
            Icon::ArrowUp => MatrixBuffer::from_data([
                0b00000000, 
                0b00010000,
                0b00111000,
                0b01111100,
                0b11111110,
                0b00010000,
                0b00010000,
                0b00010000,
            ]),
            Icon::ArrowDown => MatrixBuffer::from_data([
                0b00010000, 
                0b00010000,
                0b00010000,
                0b11111110, 
                0b01111100,
                0b00111000,
                0b00010000,
                0b00000000
            ]),
            Icon::ArrowLeft => MatrixBuffer::from_data([
                0b00001000,
                0b00011000,
                0b00111000,
                0b01111111,
                0b00111000,
                0b00011000,
                0b00001000,
                0b00000000,
            ]),
            Icon::ArrowRight => MatrixBuffer::from_data([
                0b00001000,
                0b00001100,
                0b00001110,
                0b11111111,
                0b00001110,
                0b00001100,
                0b00001000,
                0b00000000,
            ]),
            Icon::Checkmark => MatrixBuffer::from_data([
                0b00000001,
                0b00000010,
                0b00000100,
                0b10001000,
                0b01010000,
                0b00100000,
                0b00000000,
                0b00000000,
            ]),
            Icon::XMark => MatrixBuffer::from_data([
                0b10000001,
                0b01000010,
                0b00100100,
                0b00011000,
                0b00011000,
                0b00100100,
                0b01000010,
                0b10000001,
            ]),
            Icon::MusicNote => MatrixBuffer::from_data([
                0b00011100,
                0b00010100,
                0b00011100,
                0b00010100,
                0b11010100,
                0b11110100,
                0b01110000,
                0b00100000,
            ]),
            Icon::Circle => MatrixBuffer::from_data([
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
