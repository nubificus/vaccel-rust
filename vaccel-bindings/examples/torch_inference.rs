// SPDX-License-Identifier: Apache-2.0

mod utilities;

use env_logger::Env;
use log::info;
use std::path::PathBuf;
use vaccel::{
    ffi,
    ops::{torch, ModelInitialize, ModelRun},
    Resource, Session,
};

fn main() -> utilities::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    // Create session
    let mut sess = Session::new(0)?;
    info!("New session {}", sess.id());

    let path = vec![PathBuf::from("./examples/files/torch/cnn_trace.pt")
        .to_string_lossy()
        .to_string()];
    let mut model = Resource::new(&path, ffi::VACCEL_RESOURCE_MODEL)?;
    info!("New model {}", model.as_ref().id());

    // Register model with session
    model.as_mut().register(&mut sess)?;
    info!(
        "Registered model {} with session {}",
        model.as_ref().id(),
        sess.id()
    );

    // Prepare data
    let run_options = torch::Buffer::new(&[]); // vaccel torch buffer with data and size
                                               // TODO: in_tensor setting, use random inputs here,
                                               // but should be images instead
    let in_tensor =
        torch::Tensor::<f32>::new(&[3 * 224 * 224])?.with_data(&[1.0; 3 * 224 * 224])?;
    info!("in_tensor dim: {}", in_tensor.nr_dims());

    let mut sess_args = torch::InferenceArgs::new();

    sess_args.set_run_options(&run_options);
    sess_args.add_input(&in_tensor);
    sess_args.set_nr_outputs(1);

    let mut torch_model = torch::Model::new(model.as_mut());
    // Run inference
    let result = torch_model.as_mut().run(&mut sess, &mut sess_args)?;
    match result.get_output::<f32>(0) {
        Ok(out) => {
            println!("Success");
            println!(
                "Output tensor => type:{:?} nr_dims:{}",
                out.data_type(),
                out.nr_dims()
            );
            for i in 0..out.nr_dims() {
                println!("dim[{}]: {}", i, out.dim(i as usize).unwrap());
            }
        }
        Err(e) => println!("Torch JitLoadForward failed: '{}'", e),
    }

    model.as_mut().unregister(&mut sess)?;
    info!(
        "Unregistered model {} from session {}",
        model.as_ref().id(),
        sess.id()
    );

    // No need to release model explicitly, `release()` will be run on drop
    //model.as_mut().release()?;

    // No need to release session explicitly, `release()` will be run on drop
    //sess.release()?;

    Ok(())
}
