// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use log::error;
use std::ptr;
use vaccel::{
    c_pointer_to_mut_slice, ffi,
    profiling::{Profiler, ProfilerManager},
    VaccelId,
};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::AgentServiceClient;
use vaccel_rpc_proto::profiling::ProfilingRequest;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::AgentServiceClient;

impl AsRef<ProfilerManager> for VaccelRpcClient {
    fn as_ref(&self) -> &ProfilerManager {
        &self.profiler_manager
    }
}

impl VaccelRpcClient {
    pub const TIMERS_PREFIX: &'static str = "vaccel-rpc-client";

    pub fn get_profiler(&mut self, sess_id: i64) -> Result<Profiler> {
        let ctx = ttrpc::context::Context::default();

        let req = ProfilingRequest {
            session_id: sess_id,
            ..Default::default()
        };

        let resp = self.execute(AgentServiceClient::get_profiler, ctx, &req)?;

        Ok(resp.profiler.unwrap_or_default().into())
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_get_prof_regions(
    client_ptr: *mut VaccelRpcClient,
    sess_id: ffi::vaccel_id_t,
    regions_ptr: *mut ffi::vaccel_prof_region,
    nr_regions: usize,
    region_name_max: usize,
) -> usize {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return 0,
    };

    let sess_vaccel_id = match VaccelId::try_from(sess_id) {
        Ok(id) => id,
        Err(e) => {
            let err = Error::from(e);
            error!("{}", err);
            return 0;
        }
    };

    let _ret = match client.get_profiler(sess_vaccel_id.into()) {
        Ok(agent_profiler) => {
            client
                .profiler_manager
                .merge_profiler(sess_vaccel_id, agent_profiler);
            ffi::VACCEL_OK
        }
        Err(_) => return 0,
    };

    let regions_len = client
        .profiler_manager
        .get(sess_vaccel_id)
        .map_or(0, |p| p.len());

    if nr_regions == 0 {
        return regions_len;
    }

    let regions_ref = c_pointer_to_mut_slice(regions_ptr, nr_regions).unwrap_or(&mut []);

    if let Some(client_profiler) = client
        .profiler_manager
        .get(sess_vaccel_id)
        .and_then(|p| p.to_ffi())
    {
        for (w, (rk, rv)) in regions_ref.iter_mut().zip(client_profiler.iter()) {
            let n = rk.as_str();
            let n_len = if n.len() < region_name_max {
                n.len()
            } else {
                region_name_max
            };
            let cn = std::ffi::CString::new(&n[0..n_len]).unwrap();
            unsafe {
                ptr::copy_nonoverlapping(
                    cn.as_c_str().as_ptr(),
                    w.name as *mut _,
                    cn.to_bytes_with_nul().len(),
                );
            }

            let cnt = if rv.len() > w.size {
                println!(
                        "Warning: Not all profiling region samples can be returned (allocated: {} vs total: {})",
                        w.size,
                        rv.len()
                    );
                w.size
            } else {
                rv.len()
            };
            unsafe {
                ptr::copy_nonoverlapping(rv.as_ptr(), w.samples, cnt);
            }
            w.nr_entries = cnt;
        }
    } else {
        return 0;
    }

    regions_len
}
