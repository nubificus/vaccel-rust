// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use vaccel::Error as VaccelError;

pub enum Error {
    Vaccel(VaccelError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Vaccel(err) => write!(f, "vAccel runtime error: {}", err),
        }
    }
}

impl From<VaccelError> for Error {
    fn from(error: VaccelError) -> Self {
        Error::Vaccel(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
