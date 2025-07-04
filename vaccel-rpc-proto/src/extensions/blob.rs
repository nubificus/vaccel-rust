// SPDX-License-Identifier: Apache-2.0

use crate::resource::{Blob, BlobType};
use protobuf::Enum;
use vaccel::{Blob as VaccelBlob, BlobType as VaccelBlobType, Error, Result};

impl TryFrom<&Blob> for VaccelBlob {
    type Error = Error;

    fn try_from(blob: &Blob) -> Result<Self> {
        let mut vaccel_blob = Self::from_buf(blob.data.to_owned(), &blob.name, None, false)?;

        vaccel_blob.set_type(VaccelBlobType::from(blob.type_.value() as u32));
        Ok(vaccel_blob)
    }
}

impl TryFrom<Blob> for VaccelBlob {
    type Error = Error;

    fn try_from(blob: Blob) -> Result<Self> {
        let mut vaccel_blob = Self::from_buf(blob.data, &blob.name, None, false)?;

        vaccel_blob.set_type(VaccelBlobType::from(blob.type_.value() as u32));
        Ok(vaccel_blob)
    }
}

impl TryFrom<&VaccelBlob> for Blob {
    type Error = Error;

    fn try_from(vaccel: &VaccelBlob) -> Result<Self> {
        Ok(Self {
            type_: BlobType::from_i32(u32::from(vaccel.type_()) as i32)
                .unwrap()
                .into(),
            name: vaccel.name()?,
            data: vaccel.data().unwrap_or(&[]).to_vec(),
            size: vaccel.size().try_into().map_err(|e| {
                Error::ConversionFailed(format!(
                    "Could not convert blob `size` to proto `size` [{}]",
                    e
                ))
            })?,
            ..Default::default()
        })
    }
}

impl TryFrom<VaccelBlob> for Blob {
    type Error = Error;

    fn try_from(vaccel: VaccelBlob) -> Result<Self> {
        Self::try_from(&vaccel)
    }
}
