// SPDX-License-Identifier: Apache-2.0

use crate::vaccel::Status;
use vaccel::{ops::tf::lite::Status as VaccelStatus, Error, Result};

impl TryFrom<&Status> for VaccelStatus {
    type Error = Error;

    fn try_from(status: &Status) -> Result<Self> {
        Ok(Self(
            status
                .code
                .try_into()
                .map_err(|e| Error::ConversionFailed(format!("{}", e)))?,
        ))
    }
}

impl TryFrom<Status> for VaccelStatus {
    type Error = Error;

    fn try_from(status: Status) -> Result<Self> {
        Self::try_from(&status)
    }
}

impl From<&VaccelStatus> for Status {
    fn from(vaccel: &VaccelStatus) -> Self {
        Self {
            code: u8::from(*vaccel) as u32,
            ..Default::default()
        }
    }
}

impl From<VaccelStatus> for Status {
    fn from(vaccel: VaccelStatus) -> Self {
        Self {
            code: u8::from(vaccel) as u32,
            ..Default::default()
        }
    }
}
