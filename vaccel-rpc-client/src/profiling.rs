// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "async")]
use crate::asynchronous::client::VaccelRpcClient;
#[cfg(not(feature = "async"))]
use crate::sync::client::VaccelRpcClient;
use crate::{Error, Result};
use std::{collections::BTreeMap, ptr};
use vaccel::{c_pointer_to_mut_slice, ffi, profiling::ProfRegions};
#[cfg(feature = "async")]
use vaccel_rpc_proto::asynchronous::agent_ttrpc::RpcAgentClient;
use vaccel_rpc_proto::profiling::ProfilingRequest;
#[cfg(not(feature = "async"))]
use vaccel_rpc_proto::sync::agent_ttrpc::RpcAgentClient;

impl VaccelRpcClient {
    pub const TIMERS_PREFIX: &'static str = "vaccel-client";

    pub fn timer_start(&mut self, sess_id: i64, name: &str) {
        self.timers
            .entry(sess_id)
            .or_insert_with(|| ProfRegions::new(Self::TIMERS_PREFIX))
            .start(name);
    }

    pub fn timer_stop(&mut self, sess_id: i64, name: &str) {
        self.timers
            .entry(sess_id)
            .or_insert_with(|| ProfRegions::new(Self::TIMERS_PREFIX))
            .stop(name);
    }

    pub fn timers_extend(&mut self, sess_id: i64, extra: ProfRegions) {
        self.timers
            .entry(sess_id)
            .or_insert_with(|| ProfRegions::new(Self::TIMERS_PREFIX))
            .extend(extra);
    }

    pub fn timers_get_len(&self, sess_id: i64) -> usize {
        self.timers
            .entry(sess_id)
            .or_insert_with(|| ProfRegions::new(Self::TIMERS_PREFIX))
            .len()
    }

    pub fn timers_get_ffi(
        &self,
        sess_id: i64,
    ) -> Option<BTreeMap<String, Vec<ffi::vaccel_prof_sample>>> {
        self.timers
            .entry(sess_id)
            .or_insert_with(|| ProfRegions::new(Self::TIMERS_PREFIX))
            .get_ffi()
    }

    pub fn get_timers(&mut self, sess_id: i64) -> Result<ProfRegions> {
        let ctx = ttrpc::context::Context::default();

        let req = ProfilingRequest {
            session_id: sess_id,
            ..Default::default()
        };

        let mut resp = self.execute(RpcAgentClient::get_timers, ctx, &req)?;

        match resp.result.take() {
            Some(r) => Ok(r.into()),
            None => Err(Error::Undefined),
        }
    }
}

/// # Safety
///
/// `client_ptr` must be a valid pointer to an object obtained by
/// `create_client()`.
#[no_mangle]
pub unsafe extern "C" fn vaccel_rpc_client_get_timers(
    client_ptr: *mut VaccelRpcClient,
    sess_id: ffi::vaccel_id_t,
    timers_ptr: *mut ffi::vaccel_prof_region,
    nr_timers: usize,
    max_timer_name: usize,
) -> usize {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as usize,
    };

    let _ret = match client.get_timers(sess_id) {
        Ok(agent_timers) => {
            client.timers_extend(sess_id, agent_timers);
            ffi::VACCEL_OK
        }
        Err(_) => ffi::VACCEL_EINVAL,
    };

    if nr_timers == 0 {
        return client.timers_get_len(sess_id);
    }

    let timers_ref = c_pointer_to_mut_slice(timers_ptr, nr_timers).unwrap_or(&mut []);

    if let Some(client_timers) = client.timers_get_ffi(sess_id) {
        for (w, (rk, rv)) in timers_ref.iter_mut().zip(client_timers.iter()) {
            let n = rk.as_str();
            let n_len = if n.len() < max_timer_name {
                n.len()
            } else {
                max_timer_name
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
                        "Warning: Not all client timer samples can be returned (allocated: {} vs total: {})",
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
    };

    0
}
