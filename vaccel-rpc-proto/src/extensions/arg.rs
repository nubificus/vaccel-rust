// SPDX-License-Identifier: Apache-2.0

use crate::genop::{Arg, ArgType};
use protobuf::Enum;
use vaccel::{Arg as VaccelArg, ArgType as VaccelArgType, Error, Result};

impl Arg {
    pub fn try_from_vaccel_unallocated(vaccel: &VaccelArg) -> Result<Self> {
        Ok(Self {
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
            unallocated: true,
            ..Default::default()
        })
    }
}

impl TryFrom<&Arg> for VaccelArg {
    type Error = Error;

    fn try_from(arg: &Arg) -> Result<Self> {
        if arg.unallocated {
            return Err(Error::ConversionFailed(
                "Cannot convert an unallocated proto `Arg` ref to `Arg`".to_string(),
            ));
        }
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
        let buf = if arg.unallocated {
            if arg.size == 0 {
                return Err(Error::ConversionFailed(
                    "Could not convert proto `Arg` to `Arg`: Arg is unallocated but size=0"
                        .to_string(),
                ));
            }
            vec![0u8; arg.size as usize]
        } else {
            if arg.size as usize != arg.buf.len() {
                return Err(Error::ConversionFailed(format!(
                    "Could not convert proto `Arg` to `Arg`: Incorrect size; expected {} got {}",
                    arg.buf.len(),
                    arg.size,
                )));
            }
            arg.buf
        };
        Self::from_buf(
            buf,
            VaccelArgType::from(arg.arg_type.value() as u32),
            arg.custom_type_id,
        )
    }
}

impl TryFrom<&VaccelArg> for Arg {
    type Error = Error;

    fn try_from(vaccel: &VaccelArg) -> Result<Self> {
        let buf = vaccel.buf().unwrap_or(&[]);
        Ok(Self {
            buf: buf.to_vec(),
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
            unallocated: buf.is_empty(),
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
