// SPDX-License-Identifier: Apache-2.0

use crate::{ttrpc_error, vaccel_error, VaccelRpcAgent};
use log::info;
use vaccel::{ops::genop, profiling::ProfRegions};
use vaccel_rpc_proto::genop::{GenopRequest, GenopResponse, GenopResult};

impl VaccelRpcAgent {
    pub(crate) fn do_genop(&self, mut req: GenopRequest) -> ttrpc::Result<GenopResponse> {
        let mut sess = self
            .sessions
            .get_mut(&req.session_id.into())
            .ok_or_else(|| {
                ttrpc_error(ttrpc::Code::INVALID_ARGUMENT, "Unknown session".to_string())
            })?;

        let mut timers = self
            .timers
            .entry(req.session_id.into())
            .or_insert_with(|| ProfRegions::new("vaccel-agent"));
        timers.start("genop > read_args");
        let mut read_args: Vec<genop::GenopArg> =
            req.read_args.iter_mut().map(|e| e.into()).collect();
        timers.stop("genop > read_args");

        timers.start("genop > write_args");
        let mut write_args: Vec<genop::GenopArg> =
            req.write_args.iter_mut().map(|e| e.into()).collect();
        timers.stop("genop > write_args");

        info!("Genop session {}", sess.id());
        timers.start("genop > sess.genop");
        let response = match sess.genop(
            read_args.as_mut_slice(),
            write_args.as_mut_slice(),
            &mut timers,
        ) {
            Ok(_) => {
                let mut res = GenopResult::new();
                res.write_args = write_args.iter().map(|e| e.into()).collect();
                let mut resp = GenopResponse::new();
                resp.set_result(res);
                resp
            }
            Err(e) => {
                let mut resp = GenopResponse::new();
                resp.set_error(vaccel_error(e));
                resp
            }
        };
        timers.stop("genop > sess.genop");

        //timers.print();
        //timers.print_total();

        Ok(response)
    }
}
