use super::{Buffer, Code, DataType, Tensor, TensorAny, TensorType};
use crate::{ffi, ops::InferenceModel, resources::SingleModel, Error, Result, Session};
use protobuf::Enum;
use protocols::torch::{TorchDataType, TorchTensor};

pub struct InferenceArgs {
    // Do we have const here?
    run_options: *const ffi::vaccel_torch_buffer,
    in_tensors: Vec<*const ffi::vaccel_torch_tensor>,
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
            run_options: std::ptr::null::<ffi::vaccel_torch_buffer>()
                as *const ffi::vaccel_torch_buffer,
            in_tensors: vec![],
            nr_outputs: 0,
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

    pub fn set_nr_outputs(&mut self, nr_outputs: i32) {
        self.nr_outputs = nr_outputs;
    }
}

pub struct InferenceResult {
    out_tensors: Vec<*mut ffi::vaccel_torch_tensor>,
    // Do we need a torch::status here?
}

impl InferenceResult {
    pub fn new(len: usize) -> Self {
        let out_tensors = vec![std::ptr::null_mut(); len];

        InferenceResult { out_tensors }
    }

    pub fn from_vec(tensors: Vec<*mut ffi::vaccel_torch_tensor>) -> Self {
        InferenceResult {
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

impl InferenceModel<InferenceArgs, InferenceResult> for SingleModel {
    type LoadResult = ();

    fn run(&mut self, sess: &mut Session, args: &mut InferenceArgs) -> Result<InferenceResult> {
        let mut result = InferenceResult::new(args.in_tensors.len());

        match unsafe {
            // sess, model, run_options, in_tensor, nr_read, out_tensors, nr_write
            ffi::vaccel_torch_jitload_forward(
                sess.inner_mut(),
                self.inner_mut(),
                args.run_options, //.as_ptr() as *mut ffi::vaccel_torch_buffer,
                //args.in_tensors, //as *mut *mut ffi::vaccel_torch_tensor,
                args.in_tensors.as_ptr() as *mut *mut ffi::vaccel_torch_tensor,
                args.in_tensors.len() as i32,
                result.out_tensors.as_mut_ptr(),
                args.nr_outputs,
            ) as u32
        } {
            ffi::VACCEL_OK => Ok(result),
            err => Err(Error::Runtime(err)),
        }
    }

    fn load(&mut self, _sess: &mut Session) -> Result<()> {
        Ok(())
    }

    fn unload(&mut self, _sess: &mut Session) -> Result<()> {
        Ok(())
    }
}
