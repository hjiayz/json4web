#![no_std]
extern crate alloc;
pub mod de;
mod error;
pub mod ser;

pub use error::{JsonError as Error, Result};
