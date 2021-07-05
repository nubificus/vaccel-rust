use crate::ffi;
use crate::tensorflow::{Code, DataType};
use crate::{Error, Result};

use protobuf::ProtobufEnum;
use protocols::tensorflow::{TFDataType, TFTensor};

use std::ops::{Deref, DerefMut};

pub struct Tensor<T: TensorType> {
    inner: *mut ffi::vaccel_tf_tensor,
    dims: Vec<u64>,
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

fn product(values: &[u64]) -> u64 {
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
    pub fn new(dims: &[u64]) -> Self {
        let dims = Vec::from(dims);
        let data_count = product(&dims) as usize;
        let mut data = Vec::with_capacity(data_count);
        data.resize(data_count, T::zero());

        let inner = unsafe {
            ffi::vaccel_tf_tensor_new(
                dims.len() as i32,
                dims.as_ptr() as *mut _,
                T::data_type().to_int(),
            )
        };

        unsafe {
            ffi::vaccel_tf_tensor_set_data(
                inner,
                data.as_ptr() as *mut _,
                (data.len() * std::mem::size_of::<T>()) as u64,
            )
        };

        Tensor {
            inner,
            dims,
            data_count,
            data,
        }
    }

    pub unsafe fn from_vaccel_tensor(tensor: *mut ffi::vaccel_tf_tensor) -> Result<Tensor<T>> {
        if tensor.is_null() {
            return Err(Error::InvalidArgument);
        }

        if DataType::from_int((*tensor).data_type) != T::data_type() {
            return Err(Error::InvalidArgument);
        }

        let dims = std::slice::from_raw_parts((*tensor).dims as *mut _, (*tensor).nr_dims as usize);

        let data_count = product(&dims) as usize;

        let ptr = ffi::vaccel_tf_tensor_get_data(tensor);
        let data = if ptr.is_null() {
            let mut data = Vec::with_capacity(data_count);
            data.resize(data_count, T::zero());
            data
        } else {
            let data =
                std::slice::from_raw_parts(ptr as *mut T, data_count * std::mem::size_of::<T>());
            Vec::from(data)
        };

        Ok(Tensor::<T> {
            inner: tensor,
            dims: Vec::from(dims),
            data_count,
            data,
        })
    }

    pub fn with_data(mut self, data: &[T]) -> Result<Self> {
        if data.len() != self.data_count {
            return Err(Error::InvalidArgument);
        }

        for (e, v) in self.iter_mut().zip(data) {
            e.clone_from(v);
        }

        Ok(self)
    }

    pub fn nr_dims(&self) -> u64 {
        self.dims.len() as u64
    }

    pub fn dim(&self, idx: usize) -> Result<u64> {
        if idx >= self.dims.len() {
            return Err(Error::TensorFlow(Code::OutOfRange));
        }

        Ok(self.dims[idx])
    }

    pub fn data_type(&self) -> DataType {
        T::data_type()
    }

    pub fn as_grpc(&self) -> TFTensor {
        let data = unsafe {
            std::slice::from_raw_parts((*self.inner).data as *const u8, (*self.inner).size as usize)
        };

        TFTensor {
            data: data.to_owned(),
            dims: self.dims.clone(),
            field_type: TFDataType::from_i32(self.data_type().to_int() as i32).unwrap(),
            ..Default::default()
        }
    }
}

impl<T: TensorType> Drop for Tensor<T> {
    fn drop(&mut self) {
        if self.inner.is_null() {
            return;
        }

        unsafe { ffi::vaccel_tf_tensor_destroy(self.inner) };
        self.inner = std::ptr::null_mut();
    }
}

pub trait TensorAny {
    fn inner(&self) -> *const ffi::vaccel_tf_tensor;

    fn inner_mut(&mut self) -> *mut ffi::vaccel_tf_tensor;

    fn data_type(&self) -> DataType;
}

impl<T: TensorType> TensorAny for Tensor<T> {
    fn inner(&self) -> *const ffi::vaccel_tf_tensor {
        self.inner
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_tf_tensor {
        self.inner
    }

    fn data_type(&self) -> DataType {
        T::data_type()
    }
}

impl TensorAny for TFTensor {
    fn inner(&self) -> *const ffi::vaccel_tf_tensor {
        let inner = unsafe {
            ffi::vaccel_tf_tensor_new(
                self.get_dims().len() as i32,
                self.get_dims().as_ptr() as *mut _,
                self.get_field_type().value() as u32,
            )
        };

        let size = self.get_data().len() as u64;
        let data = self.get_data().to_owned();

        unsafe { ffi::vaccel_tf_tensor_set_data(inner, data.as_ptr() as *mut libc::c_void, size) };

        std::mem::forget(data);

        inner
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_tf_tensor {
        let inner = unsafe {
            ffi::vaccel_tf_tensor_new(
                self.get_dims().len() as i32,
                self.get_dims().as_ptr() as *mut _,
                self.get_field_type().value() as u32,
            )
        };

        let size = self.get_data().len() as u64;
        let data = self.get_data().to_owned();

        unsafe { ffi::vaccel_tf_tensor_set_data(inner, data.as_ptr() as *mut libc::c_void, size) };

        std::mem::forget(data);

        inner
    }

    fn data_type(&self) -> DataType {
        DataType::from_int(self.get_field_type().value() as u32)
    }
}

impl TensorAny for *mut ffi::vaccel_tf_tensor {
    fn inner(&self) -> *const ffi::vaccel_tf_tensor {
        *self
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_tf_tensor {
        *self
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

impl From<&ffi::vaccel_tf_tensor> for TFTensor {
    fn from(tensor: &ffi::vaccel_tf_tensor) -> Self {
        unsafe {
            TFTensor {
                dims: std::slice::from_raw_parts(tensor.dims as *mut u64, tensor.nr_dims as usize)
                    .to_owned(),
                field_type: TFDataType::from_i32((*tensor).data_type as i32).unwrap(),
                data: std::slice::from_raw_parts(tensor.data as *mut u8, tensor.size as usize)
                    .to_owned(),
                ..Default::default()
            }
        }
    }
}
