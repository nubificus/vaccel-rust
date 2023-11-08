use super::client::VsockClient;
use crate::{c_pointer_to_mut_slice, Error, Result};
use protocols::profiling::ProfilingRequest;
use std::ptr;
use vaccel::{ffi, profiling::ProfRegions};

impl VsockClient {
    pub const TIMERS_PREFIX: &str = "vaccel-client";

    pub fn timer_start(&mut self, sess_id: u32, name: &str) {
        self.timers
            .lock()
            .unwrap()
            .entry(sess_id)
            .or_insert(ProfRegions::new(Self::TIMERS_PREFIX))
            .start(name);
    }

    pub fn timer_stop(&mut self, sess_id: u32, name: &str) {
        self.timers
            .lock()
            .unwrap()
            .entry(sess_id)
            .or_insert(ProfRegions::new(Self::TIMERS_PREFIX))
            .stop(name);
    }

    pub fn get_timers(&mut self, sess_id: u32) -> Result<ProfRegions> {
        let ctx = ttrpc::context::Context::default();

        let req = ProfilingRequest {
            session_id: sess_id,
            ..Default::default()
        };

        let tc = self.ttrpc_client.clone();
        let mut resp = self
            .runtime
            .block_on(async { tc.get_timers(ctx, &req).await })?;

        match resp.result.take() {
            Some(r) => Ok(r.into()),
            None => Err(Error::Undefined),
        }
    }
}

#[no_mangle]
pub extern "C" fn get_timers(
    client_ptr: *mut VsockClient,
    sess_id: u32,
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
            let mut lock = client.timers.lock().unwrap();
            let timers = lock
                .entry(sess_id)
                .or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
            //let timers = client.get_timers_entry(sess_id);
            timers.extend(agent_timers);
            ffi::VACCEL_OK
        }
        Err(_) => ffi::VACCEL_EINVAL,
    };
    let mut lock = client.timers.lock().unwrap();
    let timers = lock
        .entry(sess_id)
        .or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
    //let timers = client.get_timers_entry(sess_id);

    if nr_timers == 0 {
        return timers.len();
    }

    let timers_ref = match c_pointer_to_mut_slice(timers_ptr, nr_timers) {
        Some(slice) => slice,
        None => &mut [],
    };

    //let mut lock = client.timers.lock().unwrap();
    //let timers = lock.entry(sess_id).or_insert(ProfRegions::new(VsockClient::TIMERS_PREFIX));
    //let timers = client.get_timers_entry(sess_id);
    if let Some(client_timers) = timers.get_ffi() {
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
                    cn.to_bytes_with_nul().len() as usize,
                );
            }

            let cnt = if rv.len() > w.size {
                println!(
                        "Warning: Not all vsock timer samples can be returned (allocated: {} vs total: {})",
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
