#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

pub mod driver;
pub mod error;
pub mod registers;

// Re-exports
pub use driver::Max7219;
pub use error::Error;
pub use registers::{DecodeMode, Register};

// Additional Feature specific modules and re-exports
#[cfg(feature = "led-matrix")]
pub mod led_matrix;

#[cfg(feature = "seven-segment")]
pub mod seven_segment;

#[cfg(feature = "seven-segment")]
pub use seven_segment::SevenSegment;

#[cfg(feature = "led-matrix")]
pub use led_matrix::LedMatrix;

/// Maximum number of daisy-chained displays supported
pub const MAX_DISPLAYS: usize = 8;

/// Number of digits (0 to 7) controlled by one MAX7219
pub const NUM_DIGITS: u8 = 8;
