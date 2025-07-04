// SPDX-License-Identifier: Apache-2.0

use crate::torch::{DataType, Tensor};
use protobuf::Enum;
use vaccel::ops::{
    torch::{Tensor as VaccelTensor, TensorType},
    Tensor as TensorTrait,
};

impl<T: TensorType> From<&VaccelTensor<T>> for Tensor {
    fn from(vaccel: &VaccelTensor<T>) -> Self {
        Self {
            dims: vaccel.dims().unwrap_or(&[]).to_vec(),
            type_: DataType::from_i32(u32::from(vaccel.data_type()) as i32)
                .unwrap()
                .into(),
            data: vaccel.as_bytes().unwrap_or(&[]).to_vec(),
            ..Default::default()
        }
    }
}

impl<T: TensorType> From<VaccelTensor<T>> for Tensor {
    fn from(vaccel: VaccelTensor<T>) -> Self {
        Self::from(&vaccel)
    }
}
