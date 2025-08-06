//! Error types for MAX7219 driver

/// Errors that can occur when using the MAX7219 driver
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The specified device count is invalid (exceeds maximum allowed).
    InvalidDeviceCount,
    /// Invalid scan limit value (must be 0-7)
    InvalidScanLimit,
    /// The specified register address is not valid for the MAX7219.
    InvalidRegister,
    /// Invalid device index (exceeds configured number of devices)
    InvalidDeviceIndex,
    /// Invalid digit position (0-7 for MAX7219)
    InvalidDigit,
    /// Invalid intensity value (must be 0-15)
    InvalidIntensity,
    /// Unsupported Character
    UnsupportedChar,
    /// Buffer Error
    BufferError,
    /// SPI communication error
    SpiError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SpiError => write!(f, "SPI communication error"),
            Self::InvalidDeviceIndex => write!(f, "Invalid device index"),
            Self::InvalidDigit => write!(f, "Invalid digit"),
            Self::InvalidIntensity => write!(f, "Invalid intensity value"),
            Self::InvalidScanLimit => write!(f, "Invalid scan limit value"),
            Self::InvalidDeviceCount => write!(f, "Invalid device count"),
            Self::InvalidRegister => write!(f, "Invalid register address"),
            Self::UnsupportedChar => write!(f, "Unsupported Character"),
            Self::BufferError => write!(f, "LED Matrix buffer error"),
        }
    }
}

/// Convert any embedded-hal SPI error into a general `SpiError`.
///
/// This allows using the `?` operator with SPI operations, automatically
/// mapping their error into the driver's unified [`Error`] type.
impl<E> From<E> for Error
where
    E: embedded_hal::spi::Error,
{
    fn from(_value: E) -> Self {
        Self::SpiError
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock SPI error for testing
    #[derive(Debug)]
    struct MockSpiError;

    impl core::fmt::Display for MockSpiError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "Mock SPI error")
        }
    }

    impl embedded_hal::spi::Error for MockSpiError {
        fn kind(&self) -> embedded_hal::spi::ErrorKind {
            embedded_hal::spi::ErrorKind::Other
        }
    }

    #[test]
    fn test_error_device() {
        assert_eq!(
            format!("{}", Error::InvalidDeviceCount),
            "Invalid device count"
        );
        assert_eq!(
            format!("{}", Error::InvalidScanLimit),
            "Invalid scan limit value"
        );
        assert_eq!(
            format!("{}", Error::InvalidRegister),
            "Invalid register address"
        );
        assert_eq!(
            format!("{}", Error::InvalidDeviceIndex),
            "Invalid device index"
        );
        assert_eq!(format!("{}", Error::InvalidDigit), "Invalid digit");
        assert_eq!(
            format!("{}", Error::InvalidIntensity),
            "Invalid intensity value"
        );
        assert_eq!(
            format!("{}", Error::UnsupportedChar),
            "Unsupported Character"
        );
        assert_eq!(format!("{}", Error::BufferError), "LED Matrix buffer error");
        assert_eq!(format!("{}", Error::SpiError), "SPI communication error");
    }

    #[test]
    fn test_error_debug() {
        // Test that Debug trait is implemented and works
        let error = Error::InvalidDigit;
        let debug_output = format!("{error:?}",);
        assert!(debug_output.contains("InvalidDigit"));
    }

    #[test]
    fn test_from_spi_error() {
        let spi_error = MockSpiError;
        let error: Error = spi_error.into();
        assert_eq!(error, Error::SpiError);
    }

    #[test]
    fn test_error_partialeq() {
        // Test that all variants implement PartialEq correctly
        assert!(Error::InvalidDeviceCount.eq(&Error::InvalidDeviceCount));
        assert!(!Error::InvalidDeviceCount.eq(&Error::InvalidScanLimit));
    }
}
