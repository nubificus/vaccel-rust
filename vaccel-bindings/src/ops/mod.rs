// SPDX-License-Identifier: Apache-2.0

use crate::{Handle, Result, Session};

#[macro_use]
mod macros;

pub mod genop;
pub mod image;
pub mod noop;
pub mod tf;
pub mod torch;

pub trait Tensor {
    type Data;
    type DataType;
    type ShapeType;

    /// Returns the number of dimensions of the tensor.
    fn nr_dims(&self) -> usize;

    /// Returns the dimensions of the tensor.
    fn dims(&self) -> Result<&[Self::ShapeType]>;

    /// Returns the shape of the tensor.
    /// This is equivalent to calling the `dims` method.
    fn shape(&self) -> Result<&[Self::ShapeType]> {
        self.dims()
    }

    /// Returns the dimension at the specified index from the tensor dimensions.
    fn dim(&self, idx: usize) -> Result<Self::ShapeType>;

    /// Returns the data of the tensor.
    /// This is equivalent to calling the `as_slice` method.
    fn data(&self) -> Result<Option<&[Self::Data]>>;

    /// Returns the data of the tensor as a slice of bytes.
    fn as_bytes(&self) -> Option<&[u8]>;

    /// Returns the type of the tensor data.
    fn data_type(&self) -> Self::DataType;
}

pub trait Model<'a> {
    type TensorHandle;

    /// Creates and loads a new model.
    fn load<P>(path: P, session: &'a mut Session) -> Result<Self>
    where
        P: AsRef<str>,
        Self: Sized;

    /// Unloads the model.
    fn unload(&mut self) -> Result<()>;

    /// Runs inference on the model.
    fn run<T: Tensor + Handle<CType = Self::TensorHandle>>(
        &mut self,
        in_tensors: &[T],
    ) -> Result<Vec<T>>;

    /// Returns `true` if the model is loaded.
    fn is_loaded(&self) -> bool;
}
