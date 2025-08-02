//! Error types for MAX7219 driver

/// Errors that can occur when using the MAX7219 driver
#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// The specified display count is invalid (exceeds maximum allowed).
    InvalidDisplayCount,
    /// Invalid scan limit value (must be 0-7)
    InvalidScanLimit,
    /// The specified register address is not valid for the MAX7219.
    InvalidRegister,
    /// Invalid display index (exceeds configured number of displays)
    InvalidDisplayIndex,
    /// Invalid digit position (0-7 for MAX7219)
    InvalidDigit,
    /// Invalid intensity value (must be 0-15)
    InvalidIntensity,
    /// Unsupported Character
    UnsupportedChar,
    /// Buffer Error
    BufferError,
    /// SPI communication error
    Spi(E),
}

impl<E> core::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Spi(_) => write!(f, "SPI communication error"),
            Self::InvalidDisplayIndex => write!(f, "Invalid display index"),
            Self::InvalidDigit => write!(f, "Invalid digit"),
            Self::InvalidIntensity => write!(f, "Invalid intensity value"),
            Self::InvalidScanLimit => write!(f, "Invalid scan limit value"),
            Self::InvalidDisplayCount => write!(f, "Invalid display count"),
            Self::InvalidRegister => write!(f, "Invalid register address"),
            Self::UnsupportedChar => write!(f, "Unsupported Character"),
            Self::BufferError => write!(f, "LED Matrix buffer error"),
        }
    }
}

impl<E> From<E> for Error<E>
where
    E: embedded_hal::spi::Error,
{
    fn from(value: E) -> Self {
        Self::Spi(value)
    }
}

/// A digit value outside the valid range (0-7) was provided.
#[derive(Debug, PartialEq, Eq)]
pub enum DigitError {
    /// The digit value must be between 0 and 7.
    InvalidDigit,
}

impl<E> From<DigitError> for Error<E> {
    fn from(_: DigitError) -> Self {
        Error::InvalidDigit
    }
}

/// Errors related to matrix buffer operations.
#[derive(Debug, PartialEq, Eq)]
pub enum MatrixError {
    /// A generic error related to buffer handling (e.g., invalid data or access).
    BufferError,
}

/// Converts a `MatrixError` into the main driver `Error` type.
/// This allows matrix-specific errors to be unified under the general error system.
impl<E> From<MatrixError> for Error<E> {
    fn from(_: MatrixError) -> Self {
        Error::BufferError
    }
}
