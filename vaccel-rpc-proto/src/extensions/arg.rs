// SPDX-License-Identifier: Apache-2.0

use crate::genop::{Arg, ArgType};
use protobuf::Enum;
use vaccel::{Arg as VaccelArg, ArgType as VaccelArgType, Error, Result};

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
        Self::new(
            &arg.buf,
            VaccelArgType::from(arg.arg_type.value() as u32),
            arg.custom_type_id,
        )
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
        Self::from_buf(
            arg.buf,
            VaccelArgType::from(arg.arg_type.value() as u32),
            arg.custom_type_id,
        )
    }
}

impl TryFrom<&VaccelArg> for Arg {
    type Error = Error;

    fn try_from(vaccel: &VaccelArg) -> Result<Self> {
        Ok(Self {
            buf: vaccel.buf().unwrap_or(&[]).to_vec(),
            size: vaccel.size().try_into().map_err(|e| {
                Error::ConversionFailed(format!(
                    "Could not convert arg `size` to proto `size` [{}]",
                    e
                ))
            })?,
            arg_type: ArgType::from_i32(u32::from(vaccel.type_()) as i32)
                .unwrap()
                .into(),
            custom_type_id: vaccel.custom_type_id(),
            ..Default::default()
        })
    }
}

impl TryFrom<VaccelArg> for Arg {
    type Error = Error;

    fn try_from(vaccel: VaccelArg) -> Result<Self> {
        Self::try_from(&vaccel)
    }
}
