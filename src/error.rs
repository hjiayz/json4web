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
    Syntax,
    EndOfString,
    OutOfRange,
    ParseFloatError(ParseFloatError),
    ParseIntError(ParseIntError),
    TypeMismatch(&'static str, &'static str),
    ParseError(&'static str),
    Base64Error(DecodeError),
    Utf8Error(Utf8Error),
    NaN,
    Custom(String),
}

impl Display for JsonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            JsonError::Syntax => write!(f, "syntax error"),
            JsonError::EndOfString => write!(f, "end of string"),
            JsonError::OutOfRange => write!(f, "out of range"),
            JsonError::ParseFloatError(e) => write!(f, "parse float error : {}", e),
            JsonError::ParseIntError(e) => write!(f, "parse int error : {}", e),
            JsonError::TypeMismatch(e1, e2) => write!(f, "type mismatch : {0} and {1}", e1, e2),
            JsonError::ParseError(e) => write!(f, "{} parse error", e),
            JsonError::Base64Error(e) => write!(f, "base64 decode error : {}", e),
            JsonError::Utf8Error(e) => write!(f, "Utf8 error : {}", e),
            JsonError::NaN => write!(f, "not a number"),
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
