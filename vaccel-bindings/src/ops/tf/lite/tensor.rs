// SPDX-License-Identifier: Apache-2.0

use super::{DataType, DynTensor, TensorType};
use crate::{ffi, ops::Tensor as TensorTrait, Error, Handle, Result};
use std::ptr::{self, NonNull};

/// Typed wrapper for the `struct vaccel_tflite_tensor` C object.
#[derive(Debug, PartialEq)]
pub struct Tensor<T: TensorType> {
    inner: NonNull<ffi::vaccel_tflite_tensor>,
    owned: bool,
    _data: Option<Vec<T>>,
}

impl<T: TensorType> Tensor<T> {
    /// Creates a new `Tensor<T>` with zeroed data.
    pub fn new(dims: &[i32]) -> Result<Self> {
        let data_count = Self::calculate_data_count(dims)?;

        let mut ptr: *mut ffi::vaccel_tflite_tensor = ptr::null_mut();
        match unsafe {
            ffi::vaccel_tflite_tensor_allocate(
                &mut ptr,
                dims.len() as i32,
                dims.as_ptr(),
                T::data_type().into(),
                data_count * T::data_type().size_of(),
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| Tensor {
                inner,
                owned: true,
                _data: None,
            })
            .ok_or(Error::EmptyValue)
    }

    /// Sets the data of a new `Tensor`.
    ///
    /// This can only be used with a `Tensor` created with the `new` method.
    pub fn with_data(mut self, data: &[T]) -> Result<Self> {
        if !self.owned || self._data.is_some() || self.dims().is_err() {
            return Err(Error::InvalidArgument(
                "This method can only be used with a `Tensor` created with `new()`".to_string(),
            ));
        }

        let data_count = Self::calculate_data_count(self.dims()?)?;
        if data.len() != data_count {
            return Err(Error::InvalidArgument(format!(
                "Unexpected data length: expected {}, got {}",
                data_count,
                data.len()
            )));
        }

        let slice = self.as_mut_slice()?.ok_or_else(|| {
            Error::InvalidArgument(
                "This method can only be used with a `Tensor` created with `new()`".to_string(),
            )
        })?;

        slice.copy_from_slice(data);
        Ok(self)
    }

    /// Creates a new `Tensor<T>` by consuming a vector of existing data.
    pub fn from_data(dims: &[i32], data: Vec<T>) -> Result<Self> {
        let mut data = data;
        let data_count = Self::calculate_data_count(dims)?;
        if data.len() != data_count {
            return Err(Error::InvalidArgument(format!(
                "Unexpected data length; expected {} got {}",
                data_count,
                data.len()
            )));
        }

        let mut ptr: *mut ffi::vaccel_tflite_tensor = ptr::null_mut();
        match unsafe {
            ffi::vaccel_tflite_tensor_new(
                &mut ptr,
                dims.len() as i32,
                dims.as_ptr(),
                T::data_type().into(),
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        match unsafe {
            ffi::vaccel_tflite_tensor_set_data(
                ptr,
                data.as_mut_ptr() as *mut _,
                data.len() * T::data_type().size_of(),
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

    /// Returns the data of the `Tensor` as a slice.
    pub fn as_slice(&self) -> Result<Option<&[T]>> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.data.is_null() {
            Ok(None)
        } else {
            let data_count = self.verify_data_count()?;
            Ok(Some(unsafe {
                std::slice::from_raw_parts(inner.data as *const T, data_count)
            }))
        }
    }

    /// Returns the data of the `Tensor` as a mutable slice.
    pub fn as_mut_slice(&mut self) -> Result<Option<&mut [T]>> {
        let inner = unsafe { self.inner.as_mut() };

        if inner.data.is_null() {
            Ok(None)
        } else {
            let data_count = self.verify_data_count()?;
            Ok(Some(unsafe {
                std::slice::from_raw_parts_mut(inner.data as *mut T, data_count)
            }))
        }
    }

    /// Calculates data count from a dims slice.
    fn calculate_data_count(dims: &[i32]) -> Result<usize> {
        dims.iter().product::<i32>().try_into().map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `data_count` to `usize` [{}]", e))
        })
    }

    /// Verifies the data count matches with the `Tensor` data.
    /// Returns the data count if successful.
    fn verify_data_count(&self) -> Result<usize> {
        let dims = self
            .dims()
            .map_err(|_| Error::InvalidArgument("Dimensions are not set".to_string()))?;
        let expected_data_count = Self::calculate_data_count(dims)?;

        let expected_data_size = expected_data_count * self.data_type().size_of();
        let data_size = self.as_bytes().unwrap_or(&[]).len();
        if expected_data_size != data_size {
            return Err(Error::ConversionFailed(format!(
                "Unexpected data size; expected {} got {}",
                expected_data_size, data_size
            )));
        }

        Ok(expected_data_count)
    }

    /// Creates a `Tensor` directly from its raw components
    pub(crate) fn from_raw_parts(
        ptr: NonNull<ffi::vaccel_tflite_tensor>,
        owned: bool,
        data: Option<Vec<T>>,
    ) -> Self {
        Self {
            inner: ptr,
            owned,
            _data: data,
        }
    }

    /// Decomposes a `Tensor` into its raw components
    pub(crate) fn into_raw_parts(
        mut self,
    ) -> (NonNull<ffi::vaccel_tflite_tensor>, bool, Option<Vec<T>>) {
        let parts = (self.inner, self.owned, self._data.take());
        self.take_ownership();
        parts
    }
}

impl_component_drop!(
    Tensor<T>,
    vaccel_tflite_tensor_delete,
    inner,
    owned,
    where: T: TensorType
);

impl_component_handle!(
    Tensor<T>,
    ffi::vaccel_tflite_tensor,
    inner,
    owned,
    extra_vec_fields: {
        _data: None,
    },
    where: T: TensorType
);

impl<T: TensorType> TensorTrait for Tensor<T> {
    type Data = T;
    type DataType = DataType;
    type ShapeType = i32;

    fn nr_dims(&self) -> usize {
        match self.dims() {
            Ok(dims) => dims.len(),
            Err(_) => 0,
        }
    }

    fn dims(&self) -> Result<&[i32]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.dims.is_null() {
            Err(Error::EmptyValue)
        } else {
            Ok(unsafe { std::slice::from_raw_parts(inner.dims, inner.nr_dims as usize) })
        }
    }

    fn dim(&self, idx: usize) -> Result<i32> {
        let dims = self.dims()?;

        if idx >= dims.len() {
            return Err(Error::OutOfBounds);
        }

        Ok(dims[idx])
    }

    fn data(&self) -> Result<Option<&[T]>> {
        self.as_slice()
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.data.is_null() || inner.size == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.data as *const _, inner.size) })
        }
    }

    fn data_type(&self) -> DataType {
        T::data_type()
    }
}

impl<T: TensorType> From<Tensor<T>> for DynTensor {
    fn from(tensor: Tensor<T>) -> Self {
        let (inner, owned, data) = tensor.into_raw_parts();
        DynTensor::from_raw_parts(inner, owned, data.map(|v| bytemuck::cast_vec(v)))
    }
}
