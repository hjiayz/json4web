use base64::DecodeError;
use std::fmt::Display;
use std::io::Error as IoError;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::str::Utf8Error;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum JsonError {
    #[error("syntax error")]
    Syntax,
    #[error("end of string")]
    EndOfString,
    #[error("out of range")]
    OutOfRange,
    #[error("parse float error : {0}")]
    ParseFloatError(ParseFloatError),
    #[error("parse int error : {0}")]
    ParseIntError(ParseIntError),
    #[error("type mismatch : {0} and {1}")]
    TypeMismatch(&'static str, &'static str),
    #[error("{0} parse error")]
    ParseError(&'static str),
    #[error("base64 decode error : {0}")]
    Base64Error(DecodeError),
    #[error("IO error : {0}")]
    IoError(IoError),
    #[error("Utf8 error : {0}")]
    Utf8Error(Utf8Error),
    #[error("not a number")]
    NaN,
    #[error("custom error : {0}")]
    Custom(String),
}

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

impl From<IoError> for JsonError {
    fn from(src: IoError) -> JsonError {
        JsonError::IoError(src)
    }
}

impl From<Utf8Error> for JsonError {
    fn from(src: Utf8Error) -> JsonError {
        JsonError::Utf8Error(src)
    }
}

pub type Result<T, E = JsonError> = std::result::Result<T, E>;
