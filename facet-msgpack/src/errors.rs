use core::fmt;

#[derive(Debug)]
#[non_exhaustive]
/// Errors that can occur during MessagePack encoding/decoding operations
pub enum Error {
    /// Encountered a MessagePack type that doesn't match the expected type
    UnexpectedType,
    /// Not enough data available to decode a complete MessagePack value
    InsufficientData,
    /// The MessagePack data is malformed or corrupted
    InvalidData,
    /// Encountered a field name that isn't recognized
    UnknownField(String),
    /// Required field is missing from the input
    MissingField(String),
    /// Integer value is too large for the target type
    IntegerOverflow,
    /// Shape is not supported for deserialization
    UnsupportedShape(String),
    /// Type is not supported for deserialization
    UnsupportedType(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedType => write!(f, "Unexpected MessagePack type"),
            Error::InsufficientData => write!(f, "Insufficient data to decode"),
            Error::InvalidData => write!(f, "Invalid MessagePack data"),
            Error::UnknownField(field) => write!(f, "Unknown field: {}", field),
            Error::MissingField(field) => write!(f, "Missing required field: {}", field),
            Error::IntegerOverflow => write!(f, "Integer value too large for target type"),
            Error::UnsupportedShape(shape) => {
                write!(f, "Unsupported shape for deserialization: {}", shape)
            }
            Error::UnsupportedType(typ) => {
                write!(f, "Unsupported type for deserialization: {}", typ)
            }
        }
    }
}

impl std::error::Error for Error {}
