// SPDX-License-Identifier: Apache-2.0

use super::{DataType, Tensor, TensorType};
use crate::{ffi, ops::Tensor as TensorTrait, Error, Handle, Result};
use protobuf::Enum;
use std::convert::TryFrom;
use std::ptr::{self, NonNull};
use vaccel_rpc_proto::tensorflow::{TFLiteDataType, TFLiteTensor};

/// Untyped wrapper for the `struct vaccel_tflite_tensor` C object.
#[derive(Debug, PartialEq)]
pub struct DynTensor {
    inner: NonNull<ffi::vaccel_tflite_tensor>,
    owned: bool,
    _data: Option<Vec<u8>>,
}

impl DynTensor {
    /// Creates a new `DynTensor` with zeroed data.
    pub fn new(dims: &[i32], data_type: DataType) -> Result<Self> {
        let data_count = Self::calculate_data_count(dims)?;
        let data_type_size = data_type.try_size_of().ok_or(Error::InvalidArgument(
            "No Rust type mapping for the provided 'DataType'".to_string(),
        ))?;

        Self::new_unchecked(dims, data_type, data_count * data_type_size)
    }

    /// Creates a new `DynTensor` with zeroed data without size validation.
    ///
    /// The caller must ensure the data size is correct for the tensor.
    pub fn new_unchecked(dims: &[i32], data_type: DataType, data_size: usize) -> Result<Self> {
        let mut ptr: *mut ffi::vaccel_tflite_tensor = ptr::null_mut();
        match unsafe {
            ffi::vaccel_tflite_tensor_allocate(
                &mut ptr,
                dims.len() as i32,
                dims.as_ptr(),
                data_type.into(),
                data_size,
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| DynTensor {
                inner,
                owned: true,
                _data: None,
            })
            .ok_or(Error::EmptyValue)
    }

    /// Sets the data of a new `DynTensor`.
    ///
    /// This can only be used with a `DynTensor` created with the `new` method.
    pub fn with_data(mut self, data: &[u8]) -> Result<Self> {
        if !self.owned || self._data.is_some() {
            return Err(Error::InvalidArgument(
                "This method can only be used with a `DynTensor` created with `new()`".to_string(),
            ));
        }

        let expected_data_size = unsafe { self.inner.as_ref().size };
        if data.len() != expected_data_size {
            return Err(Error::InvalidArgument(format!(
                "Unexpected data length: expected {}, got {}",
                expected_data_size,
                data.len()
            )));
        }

        let slice = self.as_mut_bytes().ok_or_else(|| {
            Error::InvalidArgument(
                "This method can only be used with a `DynTensor` created with `new()`".to_string(),
            )
        })?;

        slice.copy_from_slice(data);
        Ok(self)
    }

    /// Creates a new `DynTensor` by consuming a vector of existing data.
    pub fn from_data(dims: &[i32], data_type: DataType, data: Vec<u8>) -> Result<Self> {
        let data_count = Self::calculate_data_count(dims)?;
        let data_type_size = data_type.try_size_of().ok_or(Error::InvalidArgument(
            "No Rust type mapping for the provided `DataType`".to_string(),
        ))?;
        if data.len() != data_count * data_type_size {
            return Err(Error::InvalidArgument(format!(
                "Unexpected data length; expected {} got {}",
                data_count * data_type_size,
                data.len()
            )));
        }

        Self::from_data_unchecked(dims, data_type, data)
    }

    /// Creates a new `DynTensor` by consuming a vector of existing data without
    /// size validation.
    ///
    /// The caller must ensure the data size is correct for the tensor.
    pub fn from_data_unchecked(dims: &[i32], data_type: DataType, data: Vec<u8>) -> Result<Self> {
        let mut data = data;
        let mut ptr: *mut ffi::vaccel_tflite_tensor = ptr::null_mut();
        match unsafe {
            ffi::vaccel_tflite_tensor_new(
                &mut ptr,
                dims.len() as i32,
                dims.as_ptr(),
                data_type.into(),
            ) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        match unsafe {
            ffi::vaccel_tflite_tensor_set_data(ptr, data.as_mut_ptr() as *mut _, data.len()) as u32
        } {
            ffi::VACCEL_OK => (),
            err => return Err(Error::Ffi(err)),
        }

        NonNull::new(ptr)
            .map(|inner| DynTensor {
                inner,
                owned: true,
                _data: Some(data),
            })
            .ok_or(Error::EmptyValue)
    }

    /// Returns the data of the `DynTensor` as a typed slice.
    pub fn as_slice<T: TensorType>(&self) -> Result<Option<&[T]>> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.data.is_null() {
            Ok(None)
        } else {
            let data_count = self.verify_data_count::<T>()?;
            Ok(Some(unsafe {
                std::slice::from_raw_parts(inner.data as *const T, data_count)
            }))
        }
    }

    /// Returns the data of the `DynTensor` as a mutable typed slice.
    pub fn as_mut_slice<T: TensorType>(&mut self) -> Result<Option<&mut [T]>> {
        let inner = unsafe { self.inner.as_mut() };

        if inner.data.is_null() {
            Ok(None)
        } else {
            let data_count = self.verify_data_count::<T>()?;
            Ok(Some(unsafe {
                std::slice::from_raw_parts_mut(inner.data as *mut T, data_count)
            }))
        }
    }

    /// Returns the data of the `DynTensor` as a mutable slice of bytes.
    fn as_mut_bytes(&mut self) -> Option<&mut [u8]> {
        let inner = unsafe { self.inner.as_mut() };

        if inner.data.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(inner.data as *mut _, inner.size) })
        }
    }

    /// Calculates data count from a dims slice.
    fn calculate_data_count(dims: &[i32]) -> Result<usize> {
        dims.iter().product::<i32>().try_into().map_err(|e| {
            Error::ConversionFailed(format!("Could not convert `data_count` to `usize` [{}]", e))
        })
    }

    /// Verifies the data count of the requested type matches with the
    /// `DynTensor` data.
    /// Returns the data count if successful.
    fn verify_data_count<T: TensorType>(&self) -> Result<usize> {
        if T::data_type() != self.data_type() {
            return Err(Error::InvalidArgument(format!(
                "Unexpected type; expected '{:?}' got '{:?}'",
                self.data_type(),
                T::data_type()
            )));
        }

        let data = self.data()?.unwrap_or(&[]);
        let dims = self
            .dims()
            .map_err(|_| Error::InvalidArgument("Dimensions are not set".to_string()))?;

        let expected_data_count = Self::calculate_data_count(dims)?;
        let data_count = data.len() / std::mem::size_of::<T>();
        if expected_data_count != data_count {
            return Err(Error::ConversionFailed(format!(
                "Unexpected data count; expected {} got {}",
                expected_data_count, data_count
            )));
        }

        Ok(expected_data_count)
    }

    /// Creates a `DynTensor` directly from its raw components
    pub(crate) fn from_raw_parts(
        ptr: NonNull<ffi::vaccel_tflite_tensor>,
        owned: bool,
        data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            inner: ptr,
            owned,
            _data: data,
        }
    }

    /// Decomposes a `DynTensor` into its raw components
    pub(crate) fn into_raw_parts(
        mut self,
    ) -> (NonNull<ffi::vaccel_tflite_tensor>, bool, Option<Vec<u8>>) {
        let parts = (self.inner, self.owned, self._data.take());
        self.take_ownership();
        parts
    }
}

impl_component_drop!(DynTensor, vaccel_tflite_tensor_delete, inner, owned);

impl_component_handle!(
    DynTensor,
    ffi::vaccel_tflite_tensor,
    inner,
    owned,
    extra_vec_fields: {
        _data: None,
    }
);

impl TensorTrait for DynTensor {
    type Data = u8;
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

    fn data(&self) -> Result<Option<&[u8]>> {
        Ok(self.as_bytes())
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        let inner = unsafe { self.inner.as_ref() };

        if inner.data.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(inner.data as *const _, inner.size) })
        }
    }

    fn data_type(&self) -> DataType {
        DataType::from(unsafe { self.inner.as_ref().data_type })
    }
}

impl<T: TensorType + Copy> TryFrom<DynTensor> for Tensor<T> {
    type Error = Error;

    fn try_from(dyn_tensor: DynTensor) -> Result<Self> {
        dyn_tensor.verify_data_count::<T>()?;

        let (inner, owned, data) = dyn_tensor.into_raw_parts();
        Ok(Tensor::<T>::from_raw_parts(
            inner,
            owned,
            data.map(|v| bytemuck::cast_vec(v)),
        ))
    }
}

impl From<&DynTensor> for TFLiteTensor {
    fn from(dyn_tensor: &DynTensor) -> Self {
        TFLiteTensor {
            dims: dyn_tensor.dims().unwrap_or(&[]).to_vec(),
            type_: TFLiteDataType::from_i32(u32::from(dyn_tensor.data_type()) as i32)
                .unwrap()
                .into(),
            data: dyn_tensor.data().unwrap().unwrap_or(&[]).to_vec(),
            ..Default::default()
        }
    }
}

impl From<DynTensor> for TFLiteTensor {
    fn from(dyn_tensor: DynTensor) -> Self {
        TFLiteTensor::from(&dyn_tensor)
    }
}

impl TryFrom<&TFLiteTensor> for DynTensor {
    type Error = Error;

    fn try_from(proto_tensor: &TFLiteTensor) -> Result<Self> {
        DynTensor::from_data_unchecked(
            &proto_tensor.dims,
            DataType::from(proto_tensor.type_.value() as u32),
            proto_tensor.data.clone(),
        )
    }
}

impl TryFrom<TFLiteTensor> for DynTensor {
    type Error = Error;

    fn try_from(proto_tensor: TFLiteTensor) -> Result<Self> {
        DynTensor::from_data_unchecked(
            &proto_tensor.dims,
            DataType::from(proto_tensor.type_.value() as u32),
            proto_tensor.data,
        )
    }
}
