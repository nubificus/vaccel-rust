#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::fmt;

pub mod genop;
pub mod image;
pub mod noop;
pub mod resource;
pub mod session;
pub mod tf_inference;
pub mod tf_model;

#[derive(Debug)]
pub enum Error {
    // Error returned to us by vAccel runtime library
    Runtime(u32),

    // We received an invalid argument
    InvalidArgument,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Runtime(err) => write!(f, "vAccel runtime error {}", err),
            Error::InvalidArgument => write!(f, "An invalid argument was given to us"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
