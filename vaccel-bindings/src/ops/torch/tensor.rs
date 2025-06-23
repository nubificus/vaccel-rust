// SPDX-License-Identifier: Apache-2.0

use super::DataType;
use crate::{ffi, Error, Handle, Result};
use protobuf::Enum;
use std::ptr::{self, NonNull};
use vaccel_rpc_proto::torch::{TorchDataType, TorchTensor};

/// Wrapper for the `struct vaccel_torch_tensor` C object.
#[derive(Debug, PartialEq)]
pub struct Tensor<T: TensorType> {
    inner: NonNull<ffi::vaccel_torch_tensor>,
    owned: bool,
    _data: Option<Vec<T>>,
}

pub trait TensorType: Default + Clone {
    /// DataType of the Tensor type
    fn data_type() -> DataType;

    /// Unit value of type
    fn one() -> Self;

    /// Zero value of type
    fn zero() -> Self;
}

impl<T: TensorType> Tensor<T> {
    /// Creates a new `Tensor<T>` with zeroed data.
    pub fn new(dims: &[i64]) -> Result<Self> {
        let data_count = Self::calculate_data_count(Some(dims))?;
        let mut data = Vec::with_capacity(data_count);
        data.resize(data_count, T::zero());

        let mut ptr: *mut ffi::vaccel_torch_tensor = ptr::null_mut();
        match unsafe {
            ffi::vaccel_torch_tensor_new(
                &mut ptr,
                dims.len() as i64,
                dims.as_ptr() as *const _,
                T::data_type().to_int(),
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        match unsafe {
            ffi::vaccel_torch_tensor_set_data(
                ptr,
                data.as_ptr() as *mut _,
                data.len() * std::mem::size_of::<T>(),
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| Tensor {
                inner,
                owned: true,
                _data: Some(data),
            })
            .ok_or(Error::EmptyValue)
    }

    /// Sets the data of a new `Tensor`.
    ///
    /// This can only be used with a `Tensor` created with the `new` method.
    pub fn with_data(self, data: &[T]) -> Result<Self> {
        if let Some(dims) = self.dims() {
            let data_count = Self::calculate_data_count(Some(dims))?;
            if data.len() != data_count {
                return Err(Error::InvalidArgument(format!(
                    "'data` length must be {}",
                    data_count
                )));
            }
        }

        match self.as_mut_slice() {
            Ok(Some(slice)) => {
                for (e, v) in slice.iter_mut().zip(data) {
                    e.clone_from(v);
                }
                Ok(self)
            }
            _ => Err(Error::InvalidArgument(
                "This method can only be used with a `Tensor` created with `new()`".to_string(),
            )),
        }
    }

    /// Returns the number of dimensions of the `Tensor`.
    pub fn nr_dims(&self) -> usize {
        match self.dims() {
            Some(dims) => dims.len(),
            None => 0,
        }
    }

    /// Returns the dimensions of the `Tensor`.
    pub fn dims(&self) -> Option<&[i64]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.dims.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.dims, inner.nr_dims as usize) })
        }
    }

    /// Returns the dimensions at the specified index from the `Tensor` dimensions.
    pub fn dim(&self, idx: usize) -> Result<i64> {
        match self.dims() {
            Some(dims) => {
                if idx >= dims.len() {
                    return Err(Error::OutOfBounds);
                }

                Ok(dims[idx])
            }
            None => Err(Error::OutOfBounds),
        }
    }

    /// Returns the data of the `Tensor`.
    ///
    /// This is equivalent to calling the `as_slice` method.
    pub fn data(&self) -> Result<Option<&[T]>> {
        self.as_slice()
    }

    /// Returns the data of the `Tensor` as a slice.
    pub fn as_slice(&self) -> Result<Option<&[T]>> {
        let inner = unsafe { self.inner.as_ref() };
        let dims = self.dims();

        if inner.data.is_null() || dims.is_none() {
            Ok(None)
        } else {
            let data_count = Self::calculate_data_count(dims)?;
            Ok(Some(unsafe {
                std::slice::from_raw_parts(inner.data as *const T, data_count)
            }))
        }
    }

    /// Returns the data of the `Tensor` as a mutable slice.
    pub fn as_mut_slice(&self) -> Result<Option<&mut [T]>> {
        let inner = unsafe { self.inner.as_ref() };
        let dims = self.dims();

        if inner.data.is_null() || dims.is_none() {
            Ok(None)
        } else {
            let data_count = Self::calculate_data_count(dims)?;
            Ok(Some(unsafe {
                std::slice::from_raw_parts_mut(inner.data as *mut T, data_count)
            }))
        }
    }

    /// Returns the data of the `Tensor` as a slice of bytes.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.data.is_null() || inner.size == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.data as *const _, inner.size) })
        }
    }

    /// Returns the type of the `Tensor` data.
    pub fn data_type(&self) -> DataType {
        T::data_type()
    }

    pub fn as_grpc(&self) -> TorchTensor {
        TorchTensor {
            data: self.as_bytes().unwrap_or(&[]).to_vec(),
            dims: self.dims().unwrap_or(&[]).to_owned(),
            type_: TorchDataType::from_i32(self.data_type().to_int() as i32)
                .unwrap()
                .into(),
            ..Default::default()
        }
    }

    /// Calculates data count from a dims slice.
    fn calculate_data_count(dims: Option<&[i64]>) -> Result<usize> {
        match dims {
            Some(d) => d.iter().product::<i64>().try_into().map_err(|e| {
                Error::ConversionFailed(format!(
                    "Could not convert `data_count` to `usize` [{}]",
                    e
                ))
            }),
            None => Ok(0),
        }
    }
}

impl_component_drop!(
    Tensor<T>,
    vaccel_torch_tensor_delete,
    inner,
    owned,
    where: T: TensorType
);

impl_component_handle!(
    Tensor<T>,
    ffi::vaccel_torch_tensor,
    inner,
    owned,
    extra_vec_fields: {
        _data: None,
    },
    where: T: TensorType
);

pub trait TensorAny {
    fn inner(&self) -> Result<*const ffi::vaccel_torch_tensor>;

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_torch_tensor>;

    fn data_type(&self) -> DataType;
}

impl<T: TensorType> TensorAny for Tensor<T> {
    fn inner(&self) -> Result<*const ffi::vaccel_torch_tensor> {
        Ok(self.as_ptr())
    }

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_torch_tensor> {
        Ok(self.as_mut_ptr())
    }

    fn data_type(&self) -> DataType {
        T::data_type()
    }
}

impl TensorAny for TorchTensor {
    fn inner(&self) -> Result<*const ffi::vaccel_torch_tensor> {
        let size = self.data.len();
        let data = &self.data;

        let mut inner: *mut ffi::vaccel_torch_tensor = std::ptr::null_mut();
        match unsafe {
            ffi::vaccel_torch_tensor_allocate(
                &mut inner,
                self.dims.len() as i64,
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

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_torch_tensor> {
        let size = self.data.len();
        let data = &self.data;

        let mut inner: *mut ffi::vaccel_torch_tensor = std::ptr::null_mut();
        match unsafe {
            ffi::vaccel_torch_tensor_allocate(
                &mut inner,
                self.dims.len() as i64,
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

impl TensorAny for *mut ffi::vaccel_torch_tensor {
    fn inner(&self) -> Result<*const ffi::vaccel_torch_tensor> {
        Ok(*self)
    }

    fn inner_mut(&mut self) -> Result<*mut ffi::vaccel_torch_tensor> {
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
        DataType::Float
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

/*
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
*/

/*
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

impl TensorType for u64 {
    fn data_type() -> DataType {
        DataType::UInt64
    }

    fn one() -> Self {
        1u64
    }

    fn zero() -> Self {
        0u64
    }
}
*/
/*
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
*/

impl TryFrom<&ffi::vaccel_torch_tensor> for TorchTensor {
    type Error = Error;

    fn try_from(tensor: &ffi::vaccel_torch_tensor) -> Result<Self> {
        let nr_dims = tensor.nr_dims.try_into().map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `nr_dims` to `usize` [{}]", e))
        })?;
        unsafe {
            Ok(TorchTensor {
                dims: std::slice::from_raw_parts(tensor.dims as *mut _, nr_dims).to_owned(),
                type_: TorchDataType::from_i32(tensor.data_type as i32)
                    .unwrap()
                    .into(),
                data: std::slice::from_raw_parts(tensor.data as *mut u8, tensor.size).to_owned(),
                ..Default::default()
            })
        }
    }
}
