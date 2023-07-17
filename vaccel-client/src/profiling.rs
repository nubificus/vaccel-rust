use crate::{client::VsockClient, Result};
use protocols::profiling::ProfilingRequest;
use std::{ptr, slice};
use vaccel::{ffi, profiling::ProfRegions};

impl VsockClient {
    pub fn get_timers_entry(&mut self, sess_id: u32) -> &mut ProfRegions {
        self.timers
            .entry(sess_id)
            .or_insert(ProfRegions::new("vaccel-client"))
    }

    pub fn get_timers(&mut self, sess_id: u32) -> Result<ProfRegions> {
        let ctx = ttrpc::context::Context::default();

        let req = ProfilingRequest {
            session_id: sess_id,
            ..Default::default()
        };

        let mut resp = self.ttrpc_client.get_timers(ctx, &req)?;
        Ok(resp.take_result().into())
    }
}

#[no_mangle]
pub extern "C" fn get_timers(
    client_ptr: *mut VsockClient,
    sess_id: u32,
    timers_ptr: *mut ffi::vaccel_prof_region,
    nr_timers: usize,
) -> usize {
    let client = match unsafe { client_ptr.as_mut() } {
        Some(client) => client,
        None => return ffi::VACCEL_EINVAL as usize,
    };

    let _ret = match client.get_timers(sess_id) {
        Ok(agent_timers) => {
            let timers = client.get_timers_entry(sess_id);
            timers.extend(agent_timers);
            ffi::VACCEL_OK
        }
        Err(_) => ffi::VACCEL_EINVAL,
    };
    let timers = client.get_timers_entry(sess_id);

    if nr_timers == 0 {
        return timers.len();
    }

    let timers_ref = match c_pointer_to_mut_slice(timers_ptr, nr_timers) {
        Some(slice) => slice,
        None => &mut [],
    };

    let timers = client.get_timers_entry(sess_id);
    if let Some(client_timers) = timers.get_ffi() {
        for (w, (rk, rv)) in timers_ref.iter_mut().zip(client_timers.iter()) {
            unsafe {
                let s = std::ffi::CString::new(rk.as_str()).unwrap();
                // FIXME: check size of copy, as copy_nonoverlapping
                // checks the type of src and dest and copies *count*
                // items, not bytes.
                ptr::copy_nonoverlapping(
                    s.as_c_str().as_ptr(),
                    w.name as *mut _,
                    s.to_bytes_with_nul().len() as usize,
                );
                ptr::copy_nonoverlapping(rv.as_ptr(), w.samples, rv.len());
                w.nr_entries = rv.len() as usize;
            }
        }
    };

    0
}

fn c_pointer_to_slice<'a, T>(buf: *const T, len: usize) -> Option<&'a [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts(buf, len) })
    }
}

fn c_pointer_to_mut_slice<'a, T>(buf: *mut T, len: usize) -> Option<&'a mut [T]> {
    if buf.is_null() {
        None
    } else {
        Some(unsafe { slice::from_raw_parts_mut(buf, len) })
    }
}
