// SPDX-License-Identifier: Apache-2.0

use crate::genop::Arg;
use vaccel::{Arg as VaccelArg, Error, Result};

impl TryFrom<&Arg> for VaccelArg {
    type Error = Error;

    fn try_from(arg: &Arg) -> Result<Self> {
        if arg.size as usize != arg.buf.len() {
            return Err(Error::ConversionFailed(format!(
                "Could not convert proto `Arg` to `Arg`: Incorrect size; expected {} got {}",
                arg.buf.len(),
                arg.size,
            )));
        }
        Self::new(arg.buf.to_owned(), arg.argtype)
    }
}

impl TryFrom<Arg> for VaccelArg {
    type Error = Error;

    fn try_from(arg: Arg) -> Result<Self> {
        if arg.size as usize != arg.buf.len() {
            return Err(Error::ConversionFailed(format!(
                "Could not convert proto `Arg` to `Arg`: Incorrect size; expected {} got {}",
                arg.buf.len(),
                arg.size,
            )));
        }
        Self::new(arg.buf, arg.argtype)
    }
}

impl From<&VaccelArg> for Arg {
    fn from(vaccel: &VaccelArg) -> Self {
        Self {
            buf: vaccel.buf().unwrap_or(&[]).to_vec(),
            size: vaccel.size() as u32,
            argtype: vaccel.argtype(),
            ..Default::default()
        }
    }
}

impl From<VaccelArg> for Arg {
    fn from(vaccel: VaccelArg) -> Self {
        Self::from(&vaccel)
    }
}
