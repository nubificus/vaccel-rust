// SPDX-License-Identifier: Apache-2.0

use crate::tf::{DataType, Tensor};
use protobuf::Enum;
use vaccel::{
    ops::{
        tf::{DataType as VaccelDataType, DynTensor},
        Tensor as TensorTrait,
    },
    Error, Result,
};

impl TryFrom<&Tensor> for DynTensor {
    type Error = Error;

    fn try_from(tensor: &Tensor) -> Result<Self> {
        Self::from_data_unchecked(
            &tensor.dims,
            VaccelDataType::from(tensor.type_.value() as u32),
            tensor.data.clone(),
        )
    }
}

impl TryFrom<Tensor> for DynTensor {
    type Error = Error;

    fn try_from(tensor: Tensor) -> Result<Self> {
        Self::from_data_unchecked(
            &tensor.dims,
            VaccelDataType::from(tensor.type_.value() as u32),
            tensor.data,
        )
    }
}

impl From<&DynTensor> for Tensor {
    fn from(dyn_tensor: &DynTensor) -> Self {
        Self {
            dims: dyn_tensor.dims().unwrap_or(&[]).to_vec(),
            type_: DataType::from_i32(u32::from(dyn_tensor.data_type()) as i32)
                .unwrap()
                .into(),
            data: dyn_tensor.data().unwrap().unwrap_or(&[]).to_vec(),
            ..Default::default()
        }
    }
}

impl From<DynTensor> for Tensor {
    fn from(dyn_tensor: DynTensor) -> Self {
        Self::from(&dyn_tensor)
    }
}
