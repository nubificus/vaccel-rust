use super::{Code, Tensor, TensorAny, TensorType, Type};
use crate::{ffi, ops::InferenceModel, resources::SingleModel, Error, Result, Session};
use protobuf::Enum;
use protocols::tensorflow::{TFLiteTensor, TFLiteType, TensorflowLiteModelRunRequest};

pub struct InferenceArgs {
    in_tensors: Vec<*const ffi::vaccel_tflite_tensor>,
    nr_outputs: i32,
}

impl Default for InferenceArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceArgs {
    pub fn new() -> Self {
        InferenceArgs {
            in_tensors: vec![],
            nr_outputs: 0,
        }
    }

    pub fn add_input(&mut self, tensor: &dyn TensorAny) {
        self.in_tensors.push(tensor.inner());
    }

    pub fn set_nr_outputs(&mut self, nr_outputs: i32) {
        self.nr_outputs = nr_outputs;
    }
}

impl From<InferenceArgs> for TensorflowLiteModelRunRequest {
    fn from(args: InferenceArgs) -> Self {
        let in_tensors: Vec<TFLiteTensor> = args
            .in_tensors
            .into_iter()
            .map(|e| unsafe { e.as_ref().unwrap().into() })
            .collect();

        TensorflowLiteModelRunRequest {
            in_tensors,
            nr_outputs: args.nr_outputs,
            ..Default::default()
        }
    }
}

pub struct InferenceResult {
    out_tensors: Vec<*mut ffi::vaccel_tflite_tensor>,
    status: u8,
}

impl InferenceResult {
    pub fn new(len: usize) -> Self {
        let out_tensors = vec![std::ptr::null_mut(); len];

        InferenceResult {
            out_tensors,
            status: 0,
        }
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_tflite_tensor>) -> Self {
        InferenceResult {
            out_tensors: tensors,
            status: 0,
        }
    }

    pub fn get_output<T: TensorType>(&self, id: usize) -> Result<Tensor<T>> {
        if id >= self.out_tensors.len() {
            return Err(Error::TensorFlowLite(Code::Error));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::TensorFlowLite(Code::Error));
        }

        let inner_data_type = unsafe { Type::from_int((*t).data_type) };
        if inner_data_type != T::data_type() {
            return Err(Error::TensorFlowLite(Code::Error));
        }

        Ok(unsafe { Tensor::from_vaccel_tensor(t).unwrap() })
    }

    pub fn get_grpc_output(&self, id: usize) -> Result<TFLiteTensor> {
        if id >= self.out_tensors.len() {
            return Err(Error::TensorFlowLite(Code::Error));
        }

        let t = self.out_tensors[id];
        if t.is_null() {
            return Err(Error::TensorFlowLite(Code::Error));
        }

        unsafe {
            Ok(TFLiteTensor {
                dims: std::slice::from_raw_parts((*t).dims, (*t).nr_dims as usize).to_vec(),
                type_: TFLiteType::from_i32((*t).data_type as i32).unwrap().into(),
                data: std::slice::from_raw_parts((*t).data as *mut u8, (*t).size).to_vec(),
                ..Default::default()
            })
        }
    }
}

impl InferenceModel<InferenceArgs, InferenceResult> for SingleModel {
    type LoadResult = ();

    fn load(&mut self, sess: &mut Session) -> Result<()> {
        match unsafe { ffi::vaccel_tflite_session_load(sess.inner_mut(), self.inner_mut()) as u32 }
        {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    fn run(&mut self, sess: &mut Session, args: &mut InferenceArgs) -> Result<InferenceResult> {
        let mut result = InferenceResult::new(args.in_tensors.len());

        match unsafe {
            ffi::vaccel_tflite_session_run(
                sess.inner_mut(),
                self.inner_mut(),
                args.in_tensors.as_ptr() as *const *mut ffi::vaccel_tflite_tensor,
                args.in_tensors.len() as i32,
                result.out_tensors.as_mut_ptr(),
                args.nr_outputs,
                &mut result.status as *mut _,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            err => Err(Error::Runtime(err)),
        }
    }

    fn unload(&mut self, sess: &mut Session) -> Result<()> {
        match unsafe {
            ffi::vaccel_tflite_session_delete(sess.inner_mut(), self.inner_mut()) as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}
