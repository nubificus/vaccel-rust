// SPDX-License-Identifier: Apache-2.0

mod utilities;

use env_logger::Env;
use log::error;
use std::path::PathBuf;
use vaccel::{
    ops::{torch, Model, Tensor},
    Session,
};

fn main() -> utilities::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    // Create session
    let mut sess = Session::new()?;

    let path = PathBuf::from("./examples/files/torch/cnn_trace.pt")
        .to_string_lossy()
        .to_string();

    // Load torch model
    let mut torch_model = match torch::Model::load(path, &mut sess) {
        Ok(model) => model,
        Err(e) => {
            error!("Could not load model: {}", e);
            return Err(utilities::Error::Vaccel(e));
        }
    };

    // Run inference
    let out_tensors = match torch_model
        .run(&[torch::Tensor::<f32>::new(&[3, 224, 224])?.with_data(&[1.0; 3 * 224 * 224])?])
    {
        Ok(r) => r,
        Err(e) => {
            error!("Inference failed: {}", e);
            return Err(utilities::Error::Vaccel(e));
        }
    };
    println!("Success!");

    // View output
    let out_tensor = &out_tensors[0];
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

    // Optional: Releases the session ref
    // torch_model.unload()?;

    Ok(())
}
