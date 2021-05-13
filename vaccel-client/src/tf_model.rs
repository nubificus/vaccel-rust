use crate::client::VsockClient;
use crate::resources::VaccelResource;

use std::ffi::CStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use vaccel_bindings::{vaccel_id_t, vaccel_tf_model};
use vaccel_bindings::{VACCEL_EIO, VACCEL_ENOENT};

use protocols::resources::{CreateResourceRequest, CreateTensorflowModelRequest};

pub struct TensorFlowModel {
    data: Vec<u8>,
}

impl TensorFlowModel {
    pub fn new(path: &Path) -> Result<Self, u32> {
        let mut file = File::open(path).map_err(|_| VACCEL_EIO)?;

        let mut buff = Vec::new();
        file.read_to_end(&mut buff).map_err(|_| VACCEL_EIO)?;

        Ok(TensorFlowModel { data: buff })
    }

    pub fn from_vec(buf: &Vec<u8>) -> Result<Self, u32> {
        Ok(TensorFlowModel { data: buf.clone() })
    }
}

impl VaccelResource for TensorFlowModel {
    fn create_resource_request(self) -> Result<CreateResourceRequest, u32> {
        let mut model = CreateTensorflowModelRequest::new();
        model.set_model(self.data);

        let mut req = CreateResourceRequest::new();
        req.set_tf(model);

        Ok(req)
    }
}

pub(crate) fn create_tf_model(client: &VsockClient, model: &vaccel_tf_model) -> vaccel_id_t {
    let file = &model.file;
    if !file.path.is_null() {
        let cstr: &CStr = unsafe { CStr::from_ptr(file.path) };
        let rstr = match cstr.to_str() {
            Ok(rstr) => rstr,
            Err(_) => return -(VACCEL_ENOENT as i64),
        };

        let tf_model = match TensorFlowModel::new(Path::new(rstr)) {
            Ok(m) => m,
            Err(err) => return -(err as i64),
        };

        match client.create_resource(tf_model) {
            Ok(id) => id,
            Err(err) => return -(err as i64),
        }
    } else {
        let data =
            unsafe { Vec::from_raw_parts(file.data, file.size as usize, file.size as usize) };
        let tf_model = match TensorFlowModel::from_vec(&data) {
            Ok(m) => m,
            Err(err) => return -(err as i64),
        };
        std::mem::forget(data);

        match client.create_resource(tf_model) {
            Err(ret) => -(ret as i64),
            Ok(id) => id,
        }
    }
}
