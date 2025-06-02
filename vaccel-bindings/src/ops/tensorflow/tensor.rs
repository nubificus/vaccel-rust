// SPDX-License-Identifier: Apache-2.0

use super::DataType;
use crate::{ffi, Error, Result};
use protobuf::Enum;
use std::ops::{Deref, DerefMut};
use vaccel_rpc_proto::tensorflow::{TFDataType, TFTensor};

pub struct Tensor<T: TensorType> {
    inner: *mut ffi::vaccel_tf_tensor,
    dims: Vec<i64>,
    data_count: usize,
    data: Vec<T>,
}

pub trait TensorType: Default + Clone {
    /// DataType of the Tensor type
    fn data_type() -> DataType;

    /// Unit value of type
    fn one() -> Self;

    /// Zero value of type
    fn zero() -> Self;
}

fn product(values: &[i64]) -> i64 {
    values.iter().product()
}

impl<T: TensorType> Deref for Tensor<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        if self.inner.is_null() {
            &[]
        } else {
            let data = unsafe { (*self.inner).data } as *const T;
            unsafe { std::slice::from_raw_parts(data, self.data_count) }
        }
    }
}

impl<T: TensorType> DerefMut for Tensor<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        if self.inner.is_null() {
            &mut []
        } else {
            let data = unsafe { (*self.inner).data } as *mut T;
            unsafe { std::slice::from_raw_parts_mut(data, self.data_count) }
        }
    }
}

impl<T: TensorType> Tensor<T> {
    pub fn new(dims: &[i64]) -> Result<Self> {
        let mut dims = dims.to_vec();
        let data_count = product(&dims).try_into().unwrap();
        let mut data = Vec::with_capacity(data_count);
        data.resize(data_count, T::zero());

        let mut inner: *mut ffi::vaccel_tf_tensor = std::ptr::null_mut();
        match unsafe {
            ffi::vaccel_tf_tensor_new(
                &mut inner,
                dims.len() as i32,
                dims.as_mut_ptr() as *mut _,
                T::data_type().to_int(),
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }
        assert!(!inner.is_null());

        match unsafe {
            ffi::vaccel_tf_tensor_set_data(
                inner,
                data.as_ptr() as *mut _,
                data.len() * std::mem::size_of::<T>(),
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        Ok(Tensor {
            inner,
            dims,
            data_count,
            data,
        })
    }

    /// # Safety
    ///
    /// `tensor` is expected to be a valid pointer to an object allocated
    /// manually or by the respective vAccel function.
    pub unsafe fn from_ffi(tensor: *mut ffi::vaccel_tf_tensor) -> Result<Tensor<T>> {
        if tensor.is_null() {
            return Err(Error::InvalidArgument(
                "`tensor` cannot be `null`".to_string(),
            ));
        }

        if DataType::from_int((*tensor).data_type) != T::data_type() {
            return Err(Error::InvalidArgument(
                "Invalid `tensor.data_type`".to_string(),
            ));
        }

        let dims = std::slice::from_raw_parts((*tensor).dims as *mut _, (*tensor).nr_dims as usize);

        let data_count = product(dims).try_into().map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `data_count` to `usize` [{}]", e))
        })?;

        let data = if (*tensor).data.is_null() {
            let mut data = Vec::with_capacity(data_count);
            data.resize(data_count, T::zero());
            data
        } else {
            std::slice::from_raw_parts((*tensor).data as *mut T, data_count).to_vec()
        };

        Ok(Tensor::<T> {
            inner: tensor,
            dims: dims.to_vec(),
            data_count,
            data,
        })
    }

    pub fn with_data(mut self, data: &[T]) -> Result<Self> {
        if data.len() != self.data_count {
            return Err(Error::InvalidArgument(format!(
                "'data` length must be {}",
                self.data_count
            )));
        }

        for (e, v) in self.iter_mut().zip(data) {
            e.clone_from(v);
        }

        Ok(self)
    }

    pub fn nr_dims(&self) -> usize {
        self.dims.len()
    }

    pub fn dim(&self, idx: usize) -> Result<i64> {
        if idx >= self.dims.len() {
            return Err(Error::OutOfBounds);
        }

        Ok(self.dims[idx])
    }

    pub fn data_type(&self) -> DataType {
        T::data_type()
    }

    pub fn as_grpc(&self) -> TFTensor {
        let data = unsafe {
            std::slice::from_raw_parts((*self.inner).data as *const u8, (*self.inner).size)
        };

        TFTensor {
            data: data.to_vec(),
            dims: self.dims.to_owned(),
            type_: TFDataType::from_i32(self.data_type().to_int() as i32)
                .unwrap()
                .into(),
            ..Default::default()
        }
    }
}

impl<T: TensorType> Drop for Tensor<T> {
    fn drop(&mut self) {
        if self.inner.is_null() {
            return;
        }

        unsafe { ffi::vaccel_tf_tensor_delete(self.inner) };
        self.inner = std::ptr::null_mut();
    }
}

pub trait TensorAny {
    fn inner(&self) -> Result<*const ffi::vaccel_tf_tensor>;

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_tf_tensor>;

    fn data_type(&self) -> DataType;
}

impl<T: TensorType> TensorAny for Tensor<T> {
    fn inner(&self) -> Result<*const ffi::vaccel_tf_tensor> {
        Ok(self.inner)
    }

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_tf_tensor> {
        Ok(self.inner)
    }

    fn data_type(&self) -> DataType {
        T::data_type()
    }
}

impl TensorAny for TFTensor {
    fn inner(&self) -> Result<*const ffi::vaccel_tf_tensor> {
        let size = self.data.len();
        let data = &self.data;

        let mut inner: *mut ffi::vaccel_tf_tensor = std::ptr::null_mut();
        match unsafe {
            ffi::vaccel_tf_tensor_allocate(
                &mut inner,
                self.dims.len() as i32,
                self.dims.as_ptr() as *mut _,
                self.type_.value() as u32,
                size,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }
        assert!(!inner.is_null());

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), (*inner).data as *mut u8, size);
            (*inner).size = size;
        }

        Ok(inner)
    }

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_tf_tensor> {
        let size = self.data.len();
        let data = &self.data;

        let mut inner: *mut ffi::vaccel_tf_tensor = std::ptr::null_mut();
        match unsafe {
            ffi::vaccel_tf_tensor_allocate(
                &mut inner,
                self.dims.len() as i32,
                self.dims.as_ptr() as *mut _,
                self.type_.value() as u32,
                size,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }
        assert!(!inner.is_null());

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), (*inner).data as *mut u8, size);
            (*inner).size = size;
        }

        Ok(inner)
    }

    fn data_type(&self) -> DataType {
        DataType::from_int(self.type_.value() as u32)
    }
}

impl TensorAny for *mut ffi::vaccel_tf_tensor {
    fn inner(&self) -> Result<*const ffi::vaccel_tf_tensor> {
        Ok(*self)
    }

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_tf_tensor> {
        Ok(*self)
    }

    fn data_type(&self) -> DataType {
        DataType::from_int(unsafe { (**self).data_type })
    }
}

impl TensorType for f32 {
    fn data_type() -> DataType {
        DataType::Float
    }

    fn one() -> Self {
        1.0f32
    }

    fn zero() -> Self {
        0.0f32
    }
}

impl TensorType for f64 {
    fn data_type() -> DataType {
        DataType::Double
    }

    fn one() -> Self {
        1.0f64
    }

    fn zero() -> Self {
        0.0f64
    }
}

impl TensorType for i32 {
    fn data_type() -> DataType {
        DataType::Int32
    }

    fn one() -> Self {
        1i32
    }

    fn zero() -> Self {
        0i32
    }
}

impl TensorType for u8 {
    fn data_type() -> DataType {
        DataType::UInt8
    }

    fn one() -> Self {
        1u8
    }

    fn zero() -> Self {
        0u8
    }
}

impl TensorType for i16 {
    fn data_type() -> DataType {
        DataType::Int16
    }

    fn one() -> Self {
        1i16
    }

    fn zero() -> Self {
        0i16
    }
}

impl TensorType for i8 {
    fn data_type() -> DataType {
        DataType::Int8
    }

    fn one() -> Self {
        1i8
    }

    fn zero() -> Self {
        0i8
    }
}

impl TensorType for i64 {
    fn data_type() -> DataType {
        DataType::Int64
    }

    fn one() -> Self {
        1i64
    }

    fn zero() -> Self {
        0i64
    }
}

impl TensorType for u16 {
    fn data_type() -> DataType {
        DataType::UInt16
    }

    fn one() -> Self {
        1u16
    }

    fn zero() -> Self {
        0u16
    }
}

impl TensorType for u32 {
    fn data_type() -> DataType {
        DataType::UInt32
    }

    fn one() -> Self {
        1u32
    }

    fn zero() -> Self {
        0u32
    }
}

impl TensorType for usize {
    fn data_type() -> DataType {
        DataType::UInt64
    }

    fn one() -> Self {
        1usize
    }

    fn zero() -> Self {
        0usize
    }
}

impl TensorType for bool {
    fn data_type() -> DataType {
        DataType::Bool
    }

    fn one() -> Self {
        true
    }

    fn zero() -> Self {
        false
    }
}

impl From<&ffi::vaccel_tf_tensor> for TFTensor {
    fn from(tensor: &ffi::vaccel_tf_tensor) -> Self {
        unsafe {
            TFTensor {
                dims: std::slice::from_raw_parts(tensor.dims, tensor.nr_dims as usize).to_vec(),
                type_: TFDataType::from_i32(tensor.data_type as i32)
                    .unwrap()
                    .into(),
                data: std::slice::from_raw_parts(tensor.data as *mut u8, tensor.size).to_vec(),
                ..Default::default()
            }
        }
    }
}
