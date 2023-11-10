use crate::ffi;
use crate::session::Session;
use crate::VaccelId;
//use crate::client::VsockClient;
//use crate::resources::VaccelResource;
use crate::{Error, Result};

use protobuf::Enum;
use protocols::torch::{TorchDataType, TorchTensor};

use std::any::Any;
use std::ffi::{CStr, CString};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Code {
    Ok = 0,
    Cancelled,
    Unkown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
}

impl Code {
    pub(crate) fn to_u8(&self) -> u8 {
        match self {
            Code::Ok => 0,
            Code::Cancelled => 1,
            Code::Unkown => 2,
            Code::InvalidArgument => 3,
            Code::DeadlineExceeded => 4,
            Code::NotFound => 5,
            Code::AlreadyExists => 6,
            Code::PermissionDenied => 7,
            Code::ResourceExhausted => 8,
            Code::FailedPrecondition => 9,
            Code::Aborted => 10,
            Code::OutOfRange => 11,
            Code::Unimplemented => 12,
            Code::Internal => 13,
            Code::Unavailable => 14,
            Code::DataLoss => 15,
            Code::Unauthenticated => 16,
        }
    }
}

#[derive(Debug, PartialEq)]
/**************/
/*  DataType  */
/**************/
pub enum DataType {
    UnknownValue(u32),
    Float,
    //Double,
    Int32,
    UInt8,
    Int16,
    Int8,
    /*
    String,
    Complex64,
    Bool,
    QInt8,
    QUInt8,
    QInt32,
    BFloat16,
    QInt16,
    QUInt16,
    UInt16,
    Complex128,
    Resource,
    Variant,
    */
    Int64,
    Half,
    //UInt32,
    //UInt64,
}

impl DataType {
    pub fn to_int(&self) -> u32 {
        match self {
            DataType::UInt8 => ffi::VACCEL_TORCH_BYTE,
            DataType::Int8 => ffi::VACCEL_TORCH_CHAR,
            DataType::Int16 => ffi::VACCEL_TORCH_SHORT,
            DataType::Int32 => ffi::VACCEL_TORCH_INT,
            DataType::Int64 => ffi::VACCEL_TORCH_LONG,
            DataType::Half => ffi::VACCEL_TORCH_HALF,
            DataType::Float => ffi::VACCEL_TORCH_FLOAT,

            // DataType::Double => ffi::VACCEL_TORCH_DOUBLE,
            // DataType::String => ffi::VACCEL_TORCH_STRING,
            // DataType::Complex64 => ffi::VACCEL_TORCH_COMPLEX64,
            // DataType::Bool => ffi::VACCEL_TORCH_BOOL,
            // DataType::QInt8 => ffi::VACCEL_TORCH_QINT8,
            // DataType::QUInt8 => ffi::VACCEL_TORCH_QUINT8,
            // DataType::QInt32 => ffi::VACCEL_TORCH_QINT32,
            // DataType::BFloat16 => ffi::VACCEL_TORCH_BFLOAT16,
            // DataType::QInt16 => ffi::VACCEL_TORCH_QINT16,
            // DataType::QUInt16 => ffi::VACCEL_TORCH_QUINT16,
            // DataType::UInt16 => ffi::VACCEL_TORCH_UINT16,
            // DataType::Complex128 => ffi::VACCEL_TORCH_COMPLEX128,
            // DataType::Resource => ffi::VACCEL_TORCH_RESOURCE,
            // DataType::Variant => ffi::VACCEL_TORCH_VARIANT,
            // DataType::UInt32 => ffi::VACCEL_TORCH_UINT32,
            // DataType::UInt64 => ffi::VACCEL_TORCH_UINT64,
            DataType::UnknownValue(c) => *c,
        }
    }

    pub fn from_int(val: u32) -> DataType {
        match val {
            ffi::VACCEL_TORCH_BYTE => DataType::UInt8,
            ffi::VACCEL_TORCH_CHAR => DataType::Int8,
            ffi::VACCEL_TORCH_SHORT => DataType::Int16,
            ffi::VACCEL_TORCH_INT => DataType::Int32,
            ffi::VACCEL_TORCH_LONG => DataType::Int64,
            ffi::VACCEL_TORCH_HALF => DataType::Half,
            ffi::VACCEL_TORCH_FLOAT => DataType::Float,

            // ffi::VACCEL_TORCH_DOUBLE => DataType::Double,
            // ffi::VACCEL_TORCH_STRING => DataType::String,
            // ffi::VACCEL_TORCH_COMPLEX64 => DataType::Complex64,
            // ffi::VACCEL_TORCH_BOOL => DataType::Bool,
            // ffi::VACCEL_TORCH_QINT8 => DataType::QInt8,
            // ffi::VACCEL_TORCH_QUINT8 => DataType::QUInt8,
            // ffi::VACCEL_TORCH_QINT32 => DataType::QInt32,
            // ffi::VACCEL_TORCH_BFLOAT16 => DataType::BFloat16,
            // ffi::VACCEL_TORCH_QINT16 => DataType::QInt16,
            // ffi::VACCEL_TORCH_QUINT16 => DataType::QUInt16,
            // ffi::VACCEL_TORCH_UINT16 => DataType::UInt16,
            // ffi::VACCEL_TORCH_COMPLEX128 => DataType::Complex128,
            // ffi::VACCEL_TORCH_RESOURCE => DataType::Resource,
            // ffi::VACCEL_TORCH_VARIANT => DataType::Variant,
            // ffi::VACCEL_TORCH_UINT32 => DataType::UInt32,
            // ffi::VACCEL_TORCH_UINT64 => DataType::UInt64,
            unknown => DataType::UnknownValue(unknown),
        }
    }
}

impl Default for DataType {
    fn default() -> Self {
        DataType::Float
    }
}

#[derive(Debug, PartialEq)]
/*****************/
/*  TorchTensor  */
/*****************/
// This Tensor should be same as the vaccel tensorflow Tensor
// difference: owned - bool -> uint8_t,  dims - long long int -> int64_t
pub struct Tensor<T: TensorType> {
    inner: *mut ffi::vaccel_torch_tensor,
    dims: Vec<u32>,
    data_count: usize,
    data: Vec<T>,
}

pub trait TensorType: Default + Clone {
    // DataType - should we one to one map to the vaccelrt/src/ops/torch.c?
    fn data_type() -> DataType;

    // Unit value of type
    fn one() -> Self;

    // Zero value of type
    fn zero() -> Self;
}

/***************/
/*  TorchArgs  */
/***************/
// run_options and in_tensors args
pub struct TorchArgs {
    // Do we have const here?
    run_options: *const ffi::vaccel_torch_buffer,
    in_tensors: Vec<*const ffi::vaccel_torch_tensor>,
}

impl Default for TorchArgs {
    fn default() -> Self {
        Self::new()
    }
}
impl TorchArgs {
    pub fn new() -> Self {
        TorchArgs {
            run_options: std::ptr::null::<ffi::vaccel_torch_buffer>()
                as *const ffi::vaccel_torch_buffer,
            in_tensors: vec![],
        }
    }

    // torch::Buffer -> Buffer
    pub fn set_run_options(&mut self, run_opts: &Buffer) {
        self.run_options = run_opts.inner();
    }

    // torch::TensorAny -> TensorAny
    // TODO: &TorchTensor -> TensorAny
    pub fn add_input(&mut self, tensor: &dyn TensorAny) {
        self.in_tensors.push(tensor.inner());
    }
}

/*******************************/
/*  TorchJitloadForwardResult  */
/*******************************/
pub struct TorchJitloadForwardResult {
    out_tensors: Vec<*mut ffi::vaccel_torch_tensor>,
    // Do we need a torch::status here?
}

impl TorchJitloadForwardResult {
    pub fn new(len: usize) -> Self {
        let out_tensors = vec![std::ptr::null_mut(); len];

        TorchJitloadForwardResult { out_tensors }
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_torch_tensor>) -> Self {
        TorchJitloadForwardResult {
            out_tensors: tensors,
        }
    }

    // torch::TensorType -> TensorType
    pub fn get_output<T: TensorType>(&self, id: usize) -> Result<Tensor<T>> {
        if id >= self.out_tensors.len() {
            return Err(Error::Torch(Code::OutOfRange));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            // torch::Code -> Code
            return Err(Error::Torch(Code::Unavailable));
        }

        let inner_data_type = unsafe { DataType::from_int((*t).data_type) };
        if inner_data_type != T::data_type() {
            return Err(Error::Torch(Code::InvalidArgument));
        }

        Ok(unsafe { Tensor::from_vaccel_tensor(t).unwrap() })
    }

    pub fn get_grpc_output(&self, id: usize) -> Result<TorchTensor> {
        if id >= self.out_tensors.len() {
            return Err(Error::Torch(Code::OutOfRange));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::Torch(Code::Unavailable));
        }

        unsafe {
            Ok(TorchTensor {
                dims: std::slice::from_raw_parts((*t).dims as *mut u32, (*t).nr_dims as usize)
                    .to_owned(),
                type_: TorchDataType::from_i32((*t).data_type as i32)
                    .unwrap()
                    .into(),
                data: std::slice::from_raw_parts((*t).data as *mut u8, (*t).size).to_owned(),
                ..Default::default()
            })
        }
    }
}

// What should we do with the product func?
fn product(values: &[u32]) -> u32 {
    values.iter().product()
}

// vaccel_torch_buffer, bufferLength was required
pub struct Buffer {
    inner: *mut ffi::vaccel_torch_buffer,
    vaccel_owned: bool,
}

// Struct for the pytorch model - vaccel_torch_saved_model, model path was required
pub struct SavedModel {
    inner: *mut ffi::vaccel_torch_saved_model,
}

// TODO: original inner would be mapping to vaccel_torch_jitload_forward
pub struct TorchJitLoadForward {
    // inner: *mut ffi::vaccel_torch_jitload_forward,
    inner: *mut ffi::vaccel_torch_saved_model,
}

// TensorType, refers to TorchTensor
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
    pub fn new(dims: &[u32]) -> Self {
        let dims = Vec::from(dims);
        let data_count = product(&dims) as usize;
        let mut data = Vec::with_capacity(data_count);
        data.resize(data_count, T::zero());

        let inner = unsafe {
            ffi::vaccel_torch_tensor_new(
                dims.len() as i32,
                dims.as_ptr() as *mut _,
                T::data_type().to_int(),
            )
        };

        unsafe {
            ffi::vaccel_torch_tensor_set_data(
                inner,
                data.as_ptr() as *mut _,
                data.len() * std::mem::size_of::<T>(),
            )
        };

        Tensor {
            inner,
            dims,
            data_count,
            data,
        }
    }

    pub unsafe fn from_vaccel_tensor(tensor: *mut ffi::vaccel_torch_tensor) -> Result<Tensor<T>> {
        if tensor.is_null() {
            return Err(Error::InvalidArgument);
        }

        if DataType::from_int((*tensor).data_type) != T::data_type() {
            return Err(Error::InvalidArgument);
        }

        let dims = std::slice::from_raw_parts((*tensor).dims as *mut _, (*tensor).nr_dims as usize);

        let data_count = product(dims) as usize;

        let ptr = ffi::vaccel_torch_tensor_get_data(tensor);
        let data = if ptr.is_null() {
            let mut data = Vec::with_capacity(data_count);
            data.resize(data_count, T::zero());
            data
        } else {
            let data = std::slice::from_raw_parts(ptr as *mut T, data_count);
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

    pub fn nr_dims(&self) -> i32 {
        self.dims.len() as i32
    }

    pub fn dim(&self, idx: usize) -> Result<u32> {
        if idx >= self.dims.len() {
            return Err(Error::InvalidArgument);
        }

        Ok(self.dims[idx])
    }

    pub fn data_type(&self) -> DataType {
        T::data_type()
    }

    pub fn as_grpc(&self) -> TorchTensor {
        let data = unsafe {
            std::slice::from_raw_parts((*self.inner).data as *const u8, (*self.inner).size)
        };

        TorchTensor {
            data: data.to_owned(),
            dims: self.dims.clone(),
            type_: TorchDataType::from_i32(self.data_type().to_int() as i32)
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

        unsafe { ffi::vaccel_torch_tensor_destroy(self.inner) };
        self.inner = std::ptr::null_mut();
    }
}

pub trait TensorAny {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor;

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor;

    fn data_type(&self) -> DataType;
}

impl<T: TensorType> TensorAny for Tensor<T> {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor {
        self.inner
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor {
        self.inner
    }

    fn data_type(&self) -> DataType {
        T::data_type()
    }
}

impl TensorAny for TorchTensor {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor {
        let inner = unsafe {
            ffi::vaccel_torch_tensor_new(
                self.dims.len() as i32,
                self.dims.as_ptr() as *mut _,
                self.type_.value() as u32,
            )
        };

        let size = self.data.len();
        let data = self.data.to_owned();

        unsafe {
            ffi::vaccel_torch_tensor_set_data(inner, data.as_ptr() as *mut libc::c_void, size)
        };

        std::mem::forget(data);

        inner
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor {
        let inner = unsafe {
            ffi::vaccel_torch_tensor_new(
                self.dims.len() as i32,
                self.dims.as_ptr() as *mut _,
                self.type_.value() as u32,
            )
        };

        let size = self.data.len();
        let data = self.data.to_owned();

        unsafe {
            ffi::vaccel_torch_tensor_set_data(inner, data.as_ptr() as *mut libc::c_void, size)
        };

        std::mem::forget(data);

        inner
    }

    fn data_type(&self) -> DataType {
        DataType::from_int(self.type_.value() as u32)
    }
}

impl TensorAny for *mut ffi::vaccel_torch_tensor {
    fn inner(&self) -> *const ffi::vaccel_torch_tensor {
        *self
    }

    fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_tensor {
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

impl From<&ffi::vaccel_torch_tensor> for TorchTensor {
    fn from(tensor: &ffi::vaccel_torch_tensor) -> Self {
        unsafe {
            TorchTensor {
                dims: std::slice::from_raw_parts(tensor.dims as *mut u32, tensor.nr_dims as usize)
                    .to_owned(),
                type_: TorchDataType::from_i32(tensor.data_type as i32)
                    .unwrap()
                    .into(),
                data: std::slice::from_raw_parts(tensor.data as *mut u8, tensor.size).to_owned(),
                ..Default::default()
            }
        }
    }
}

/*------------------------------*/

impl Buffer {
    pub fn new(data: &[u8]) -> Self {
        let inner = unsafe { ffi::vaccel_torch_buffer_new(data.as_ptr() as *mut _, data.len()) };
        assert!(!inner.is_null(), "Memory allocation failure");

        Buffer {
            inner,
            vaccel_owned: false,
        }
    }

    pub unsafe fn from_vaccel_buffer(buffer: *mut ffi::vaccel_torch_buffer) -> Result<Self> {
        let mut size = Default::default();
        let data = ffi::vaccel_torch_buffer_get_data(buffer, &mut size);
        if data.is_null() || size == 0 {
            return Err(Error::InvalidArgument);
        }

        Ok(Buffer {
            inner: buffer,
            vaccel_owned: true,
        })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let mut size = Default::default();
            let ptr = ffi::vaccel_torch_buffer_get_data(self.inner, &mut size) as *const u8;
            std::slice::from_raw_parts(ptr, size)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            let mut size = Default::default();
            let ptr = ffi::vaccel_torch_buffer_get_data(self.inner, &mut size) as *mut u8;
            std::slice::from_raw_parts_mut(ptr, size)
        }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_torch_buffer {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_buffer {
        self.inner
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.vaccel_owned {
            // Data is not owned from vaccel runtime. Unset it from
            // the buffer so we avoid double free.
            let mut size = Default::default();
            unsafe { ffi::vaccel_torch_buffer_take_data(self.inner, &mut size) };
        }

        unsafe { ffi::vaccel_torch_buffer_destroy(self.inner) }
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

/*------------------------------*/

// Function for saved model - vaccel_torch_saved_model_new
// Create - SetPath - Destroy
impl Default for SavedModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SavedModel {
    // New Saved Model Object
    pub fn new() -> Self {
        SavedModel {
            inner: unsafe { ffi::vaccel_torch_saved_model_new() },
        }
    }

    /// Create a new SavedModel from a vaccel saved model type
    pub fn from_vaccel(inner: *mut ffi::vaccel_torch_saved_model) -> Self {
        SavedModel { inner }
    }

    pub fn id(&self) -> VaccelId {
        let inner = unsafe { ffi::vaccel_torch_saved_model_id(self.inner) };
        VaccelId::from(inner)
    }

    // Return True if already been initialized
    pub fn initialized(&self) -> bool {
        self.id().has_id()
    }

    pub fn destroy(&mut self) -> Result<()> {
        if !self.initialized() {
            return Ok(());
        }

        match unsafe { ffi::vaccel_torch_saved_model_destroy(self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    fn set_path(&mut self, path: &Path) -> Result<()> {
        let c_path = CString::new(path.as_os_str().to_str().ok_or(Error::InvalidArgument)?)
            .map_err(|_| Error::InvalidArgument)?;

        match unsafe {
            ffi::vaccel_torch_saved_model_set_path(self.inner, c_path.into_raw()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    // Create Resource from the exported saved model
    pub fn from_export_dir(mut self, path: &Path) -> Result<Self> {
        self.set_path(path)?;
        match unsafe { ffi::vaccel_torch_saved_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    // Set the in-memory protobuf data
    fn set_protobuf(&mut self, data: &[u8]) -> Result<()> {
        match unsafe {
            ffi::vaccel_torch_saved_model_set_model(self.inner, data.as_ptr(), data.len()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    // Create Resource from in-memory data
    pub fn from_in_memory(mut self, protobuf: &[u8]) -> Result<Self> {
        self.set_protobuf(protobuf)?;
        match unsafe { ffi::vaccel_torch_saved_model_register(self.inner) } as u32 {
            ffi::VACCEL_OK => Ok(self),
            err => Err(Error::Runtime(err)),
        }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_torch_saved_model {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_saved_model {
        self.inner
    }

    // Get the path
    pub fn get_path(&self) -> Option<PathBuf> {
        let path_str = match unsafe {
            CStr::from_ptr(ffi::vaccel_torch_saved_model_get_path(self.inner)).to_str()
        } {
            Ok(s) => s,
            Err(_) => return None,
        };

        Some(PathBuf::from(path_str))
    }

    // Get the data of the protobuf
    pub fn get_protobuf(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_torch_saved_model_get_model(self.inner, &mut size) };
        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size) })
        } else {
            None
        }
    }
}

impl crate::resource::Resource for SavedModel {
    fn id(&self) -> VaccelId {
        self.id()
    }

    fn initialized(&self) -> bool {
        self.initialized()
    }

    fn to_vaccel_ptr(&self) -> Option<*const ffi::vaccel_resource> {
        if !self.initialized() {
            None
        } else {
            let resource = unsafe { (*self.inner).resource };
            Some(resource)
        }
    }

    fn to_mut_vaccel_ptr(&self) -> Option<*mut ffi::vaccel_resource> {
        if !self.initialized() {
            None
        } else {
            let resource = unsafe { (*self.inner).resource };
            Some(resource)
        }
    }

    fn destroy(&mut self) -> Result<()> {
        self.destroy()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

/*------------------------------*/

// Function for the torch jitload
impl Default for TorchJitLoadForward {
    fn default() -> Self {
        Self::new()
    }
}

impl TorchJitLoadForward {
    pub fn new() -> Self {
        TorchJitLoadForward {
            inner: unsafe { ffi::vaccel_torch_saved_model_new() },
        }
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_torch_saved_model {
        self.inner
    }

    pub fn jitload_forward(
        &mut self,
        sess: &mut Session,
        args: &mut TorchArgs,
        model: &mut SavedModel,
    ) -> Result<TorchJitloadForwardResult> {
        let mut result = TorchJitloadForwardResult::new(args.in_tensors.len());

        match unsafe {
            // sess, model, run_options, in_tensor, nr_read, out_tensors, nr_write
            ffi::vaccel_torch_jitload_forward(
                sess.inner_mut(),
                model.inner_mut(),
                args.run_options, //.as_ptr() as *mut ffi::vaccel_torch_buffer,
                //args.in_tensors, //as *mut *mut ffi::vaccel_torch_tensor,
                args.in_tensors.as_ptr() as *mut *mut ffi::vaccel_torch_tensor,
                args.in_tensors.len() as i32,
                result.out_tensors.as_mut_ptr(),
                result.out_tensors.len() as i32,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            err => Err(Error::Runtime(err)),
        }
    }
}
