use alloc::string::String;
use alloc::string::ToString;
use base64::DecodeError;
use core::fmt::Display;
use core::num::ParseFloatError;
use core::num::ParseIntError;
use core::str::Utf8Error;
use serde::de::StdError;

#[derive(Debug)]
pub enum JsonError {
    UnexpectedEnd,
    InvalidUnicodeEscapeSequence,
    UnexpectedUnicodeEscapeSequence(u32),
    UnexpectedToken(char),
    OutOfRange,
    ParseFloatError(ParseFloatError),
    ParseIntError(ParseIntError),
    Base64Error(DecodeError),
    Utf8Error(Utf8Error),
    Custom(String),
}

impl Display for JsonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            JsonError::UnexpectedEnd => write!(f, "Unexpected end of JSON input"),
            JsonError::InvalidUnicodeEscapeSequence => write!(f, "Invalid Unicode escape sequence"),
            JsonError::UnexpectedUnicodeEscapeSequence(h) => {
                write!(f, "Unexpected Unicode escape sequence {:#08X}", h)
            }
            JsonError::UnexpectedToken(token) => write!(f, "Unexpected token {}", token),
            JsonError::OutOfRange => write!(f, "out of range"),
            JsonError::ParseFloatError(e) => write!(f, "parse float error : {}", e),
            JsonError::ParseIntError(e) => write!(f, "parse int error : {}", e),
            JsonError::Base64Error(e) => write!(f, "base64 decode error : {}", e),
            JsonError::Utf8Error(e) => write!(f, "Utf8 error : {}", e),
            JsonError::Custom(e) => write!(f, "custom error : {}", e),
        }
    }
}

impl StdError for JsonError {}

impl serde::de::Error for JsonError {
    fn custom<T: Display>(msg: T) -> Self {
        JsonError::Custom(msg.to_string())
    }
}

impl serde::ser::Error for JsonError {
    fn custom<T: Display>(msg: T) -> Self {
        JsonError::Custom(msg.to_string())
    }
}

impl From<ParseFloatError> for JsonError {
    fn from(src: ParseFloatError) -> JsonError {
        JsonError::ParseFloatError(src)
    }
}

impl From<ParseIntError> for JsonError {
    fn from(src: ParseIntError) -> JsonError {
        JsonError::ParseIntError(src)
    }
}

impl From<DecodeError> for JsonError {
    fn from(src: DecodeError) -> JsonError {
        JsonError::Base64Error(src)
    }
}

impl From<Utf8Error> for JsonError {
    fn from(src: Utf8Error) -> JsonError {
        JsonError::Utf8Error(src)
    }
}

pub type Result<T, E = JsonError> = core::result::Result<T, E>;
