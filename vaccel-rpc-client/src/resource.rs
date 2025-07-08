// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, IntoFfiResult, Result};
use log::error;
use protobuf::Enum;
use std::ffi::{c_char, c_int, CStr};
use vaccel::{c_pointer_to_slice, ffi, profiling::SessionProfiler, Blob, Handle, VaccelId};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::resource::{
    Blob as ProtoBlob, RegisterResourceRequest, ResourceType, SyncResourceRequest,
    UnregisterResourceRequest,
};
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;

impl VaccelRpcClient {
    pub fn resource_register(
        &self,
        paths: Vec<String>,
        blobs: Vec<ProtoBlob>,
        res_type: i32,
        res_id: i64,
        sess_id: i64,
    ) -> Result<i64> {
        let ctx = ttrpc::context::Context::default();
        let mut req = RegisterResourceRequest::new();
        req.paths = paths;
        req.blobs = blobs;
        req.resource_type = ResourceType::from_i32(res_type)
            .ok_or(Error::InvalidArgument("Invalid resource type".to_string()))?
            .into();
        req.resource_id = res_id;
        req.session_id = sess_id;

        let resp = self.execute(AgentServiceClient::register_resource, ctx, &req)?;

        Ok(VaccelId::try_from(resp.resource_id)?.into())
    }

    pub fn resource_unregister(&self, res_id: i64, sess_id: i64) -> Result<()> {
        let ctx = ttrpc::context::Context::default();
        let mut req = UnregisterResourceRequest::new();
        req.resource_id = res_id;
        req.session_id = sess_id;

        self.execute(AgentServiceClient::unregister_resource, ctx, &req)?;

        Ok(())
    }

    pub fn resource_sync(&self, res_id: i64) -> Result<Vec<ProtoBlob>> {
        let ctx = ttrpc::context::Context::default();
        let mut req = SyncResourceRequest::new();
        req.resource_id = res_id;

        let resp = self.execute(AgentServiceClient::sync_resource, ctx, &req)?;

        Ok(resp.blobs)
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `res_ptr` is expected to be a valid pointer to a resource
/// object allocated manually or by the respective vAccel functions.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_resource_register(
    client_ptr: *mut VaccelRpcClient,
    paths_ptr: *mut *mut c_char,
    blobs_ptr: *mut *mut ffi::vaccel_blob,
    nr_elems: usize,
    r#type: u32,
    res_id: ffi::vaccel_id_t,
    sess_id: ffi::vaccel_id_t,
) -> ffi::vaccel_id_t {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return -(ffi::VACCEL_EINVAL as ffi::vaccel_id_t),
    };

    let res_vaccel_id = match VaccelId::from_ffi(res_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return -(err.to_ffi() as ffi::vaccel_id_t);
        }
    };

    let sess_vaccel_id = match VaccelId::try_from(sess_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return -(err.to_ffi() as ffi::vaccel_id_t);
        }
    };

    let mut proto_blobs: Vec<ProtoBlob> = Vec::new();
    let mut paths: Vec<String> = Vec::new();
    if res_vaccel_id.is_none() {
        if !paths_ptr.is_null() {
            let _scope = client.profile_scope(sess_vaccel_id, "client_resource_register > paths");
            let p_slice = match c_pointer_to_slice(paths_ptr, nr_elems) {
                Some(slice) => slice,
                None => return -(ffi::VACCEL_EINVAL as ffi::vaccel_id_t),
            };
            paths = match p_slice
                .iter()
                .map(|&p| {
                    Ok(CStr::from_ptr(p)
                        .to_str()
                        .map_err(|e| {
                            Error::InvalidArgument(format!(
                                "Could not convert `paths` to `String` [{}]",
                                e
                            ))
                        })?
                        .to_string())
                })
                .collect::<Result<Vec<String>>>()
            {
                Ok(p) => p,
                Err(e) => {
                    error!("{}", e);
                    return -(e.to_ffi() as ffi::vaccel_id_t);
                }
            };
        } else {
            let _scope = client.profile_scope(sess_vaccel_id, "client_resource_register > files");
            let blobs = match c_pointer_to_slice(blobs_ptr, nr_elems) {
                Some(slice) => slice,
                None => return -(ffi::VACCEL_EINVAL as ffi::vaccel_id_t),
            };
            proto_blobs = match blobs
                .iter()
                .map(|ptr| Ok(Blob::from_ptr(*ptr)?.try_into()?))
                .collect::<Result<Vec<ProtoBlob>>>()
            {
                Ok(f) => f,
                Err(e) => {
                    error!("{}", e);
                    return -(e.to_ffi() as ffi::vaccel_id_t);
                }
            };
        }
    }

    client
        .resource_register(
            paths,
            proto_blobs,
            r#type as i32,
            res_vaccel_id.map_or(0, |id| id.into()),
            sess_vaccel_id.into(),
        )
        .into_ffi()
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_resource_unregister(
    client_ptr: *const VaccelRpcClient,
    res_id: ffi::vaccel_id_t,
    sess_id: ffi::vaccel_id_t,
) -> c_int {
    let client = match unsafe { client_ptr.as_ref() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let res_vaccel_id = match VaccelId::try_from(res_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return err.to_ffi() as c_int;
        }
    };

    let sess_vaccel_id = match VaccelId::try_from(sess_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return err.to_ffi() as c_int;
        }
    };

    client
        .resource_unregister(res_vaccel_id.into(), sess_vaccel_id.into())
        .into_ffi()
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
/// `res_ptr` is expected to be a valid pointer to a resource
/// object allocated manually or by the respective vAccel functions.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_resource_sync(
    client_ptr: *mut VaccelRpcClient,
    data_ptrs: *mut *mut u8,
    nr_elems: usize,
    res_id: ffi::vaccel_id_t,
) -> c_int {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let ptrs_slice = match c_pointer_to_slice(data_ptrs, nr_elems) {
        Some(slice) => slice,
        None => return ffi::VACCEL_EINVAL as c_int,
    };

    let res_vaccel_id = match VaccelId::try_from(res_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return err.to_ffi() as c_int;
        }
    };

    let blobs = match client.resource_sync(res_vaccel_id.into()) {
        Ok(b) => b,
        Err(e) => {
            error!("{}", e);
            return e.to_ffi() as c_int;
        }
    };

    if blobs.len() != nr_elems {
        error!(
            "Unexpected blob count; expected {} got {}",
            blobs.len(),
            nr_elems
        );
        return ffi::VACCEL_EBACKEND as c_int;
    }

    for (i, blob) in blobs.iter().enumerate() {
        let data = blob.data.as_ptr();
        let size = blob.size as usize;
        let dest = ptrs_slice[i];
        std::ptr::copy_nonoverlapping(data, dest, size);
    }

    ffi::VACCEL_OK as c_int
}
