// SPDX-License-Identifier: Apache-2.0

mod utilities;

use env_logger::Env;
use log::{error, info};
use std::path::PathBuf;
use vaccel::{
    ffi,
    ops::{tensorflow::lite as tflite, ModelInitialize, ModelLoadUnload, ModelRun},
    Resource, Session,
};

fn main() -> utilities::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    // Create session
    let mut sess = Session::new(0)?;
    info!("New session {}", sess.id());

    let path = vec![PathBuf::from("./examples/files/tf/lstm2.tflite")
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

    let mut tflite_model = tflite::Model::new(model.as_mut());
    // Load model file
    if let Err(e) = tflite_model.as_mut().load(&mut sess) {
        error!(
            "Could not load file for model {}: {}",
            model.as_ref().id(),
            e
        );

        info!("Destroying session {}", sess.id());
        sess.release()
            .expect("Could not destroy session during cleanup");

        return Err(utilities::Error::Vaccel(e));
    }

    // Prepare data for inference
    let in_tensor = tflite::Tensor::<f32>::new(&[1, 30])?.with_data(&[1.0; 30])?;

    let mut sess_args = tflite::InferenceArgs::new();
    sess_args.add_input(&in_tensor)?;
    sess_args.set_nr_outputs(1);

    // Run inference
    let mut result = tflite_model.as_mut().run(&mut sess, &mut sess_args)?;
    match result.take_output::<f32>(0) {
        Ok(out) => {
            println!("Success!");
            println!(
                "Output tensor => type:{:?} nr_dims:{}",
                out.data_type(),
                out.nr_dims()
            );
            for i in 0..out.nr_dims() {
                println!("dim[{}]: {}", i, out.dim(i).unwrap());
            }
            println!("Result Tensor :");
            for i in 0..10 {
                println!("{:.6}", out[i]);
            }
        }
        Err(e) => println!("Inference failed: '{}'", e),
    }

    tflite_model.as_mut().unload(&mut sess)?;

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
