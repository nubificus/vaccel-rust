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
    let mut sess = Session::new()?;
    info!("New session {}", sess.id());

    let path = vec![PathBuf::from("./examples/files/torch/cnn_trace.pt")
        .to_string_lossy()
        .to_string()];
    let mut model = Resource::new(&path, ffi::VACCEL_RESOURCE_MODEL)?;
    info!("New model {}", model.id());

    // Register model with session
    model.register(&mut sess)?;
    info!("Registered model {} with session {}", model.id(), sess.id());

    // Prepare data
    let in_tensor = torch::Tensor::<f32>::new(&[3, 224, 224])?.with_data(&[1.0; 3 * 224 * 224])?;

    let mut sess_args = torch::InferenceArgs::new();

    sess_args.add_input(&in_tensor)?;
    sess_args.set_nr_outputs(1);

    let mut torch_model = torch::Model::new(&mut model);
    // Run inference
    let mut result = match torch_model.run(&mut sess, &mut sess_args) {
        Ok(r) => r,
        Err(e) => {
            println!("Inference failed: {}", e);
            return Err(utilities::Error::Vaccel(e));
        }
    };

    // Get output
    let out_tensor = match result.take_output::<f32>(0) {
        Ok(tensor) => tensor,
        Err(e) => {
            println!("Failed to get output tensor: {}", e);
            return Err(utilities::Error::Vaccel(e));
        }
    };
    println!("Success!");
    println!(
        "Output tensor => type:{:?} nr_dims:{}",
        out_tensor.data_type(),
        out_tensor.nr_dims()
    );
    for i in 0..out_tensor.nr_dims() {
        println!("dim[{}]: {}", i, out_tensor.dim(i)?);
    }
    println!("Result Tensor:");
    match out_tensor.data()? {
        Some(data) => {
            for d in data.iter().take(10) {
                println!("{:.6}", d);
            }
        }
        None => println!("None"),
    };

    model.unregister(&mut sess)?;
    info!(
        "Unregistered model {} from session {}",
        model.id(),
        sess.id()
    );

    Ok(())
}
