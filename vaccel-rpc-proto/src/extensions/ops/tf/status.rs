// SPDX-License-Identifier: Apache-2.0

use crate::vaccel::Status;
use vaccel::{ops::tf::Status as VaccelStatus, Error, Result};

impl TryFrom<&Status> for VaccelStatus {
    type Error = Error;

    fn try_from(status: &Status) -> Result<Self> {
        Self::new(
            status.code.try_into().map_err(|e| {
                Error::ConversionFailed(format!("Could not convert `status.code` to `u8` [{}]", e))
            })?,
            &status.message,
        )
    }
}

impl TryFrom<Status> for VaccelStatus {
    type Error = Error;

    fn try_from(status: Status) -> Result<Self> {
        Self::try_from(&status)
    }
}

impl TryFrom<&VaccelStatus> for Status {
    type Error = Error;

    fn try_from(vaccel: &VaccelStatus) -> Result<Self> {
        Ok(Self {
            code: vaccel.code().into(),
            message: vaccel.message()?,
            ..Default::default()
        })
    }
}

impl TryFrom<VaccelStatus> for Status {
    type Error = Error;

    fn try_from(vaccel: VaccelStatus) -> Result<Self> {
        Self::try_from(&vaccel)
    }
}
