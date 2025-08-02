#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

//! MAX7219 Driver for 7-segment displays and LED matrices
//!
//! This crate provides a generic driver for the MAX7219 LED display driver
//! that supports both 7-segment displays and 8x8 LED matrices. Multiple
//! devices can be daisy-chained together.

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
