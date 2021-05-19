use env_logger::Env;
use log::{error, info};

use vaccel_bindings::resource::VaccelResource;
use vaccel_bindings::{
    vaccel_session, vaccel_tf_buffer, vaccel_tf_node, vaccel_tf_tensor, VACCEL_TF_FLOAT,
};

use std::path::Path;

extern crate utilities;
use utilities::*;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut sess = vaccel_session::new(0).map_err(|e| Error::Vaccel(e))?;
    info!("New session {}", sess.id());

    let path = Path::new("./examples/files/tf/frozen_graph.pb");

    // Create a TensorFlow model resource directly from File
    let mut model = match create_from_file(path) {
        Ok(model) => model,
        Err(err) => {
            sess.close().unwrap();
            return Err(err);
        }
    };

    // Register model with session
    if let Err(err) = register_model(&mut sess, &mut model) {
        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");
        return Err(err);
    }

    // Load model graph
    if let Err(err) = model.load_graph(&mut sess) {
        error!("Could not load graph for model {}: {}", model.id(), err);

        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");

        return Err(Error::Vaccel(err));
    }

    // Prepare data for inference
    let run_options = vaccel_tf_buffer::default();
    let in_nodes = [vaccel_tf_node::new("serving_default_input\0", 0)];
    let mut in_data = vec![1.0; 30];
    let mut dims: Vec<i64> = vec![1, 30];
    let in_tensors = [vaccel_tf_tensor::new(
        &mut in_data,
        &mut dims,
        VACCEL_TF_FLOAT,
    )];
    let out_nodes = [vaccel_tf_node::new("Identity\0", 0)];

    let (out, _) = model
        .inference(&mut sess, &run_options, &in_nodes, &in_tensors, &out_nodes)
        .map_err(|e| Error::Vaccel(e))?;

    info!("Response with {} tensors", out.len());
    for tensor in &out {
        let dims = unsafe { std::slice::from_raw_parts(tensor.dims, tensor.nr_dims as usize) };
        info!("Tensor => dims: {:?} type: {}", dims, tensor.data_type);

        let array = tensor.as_slice::<f32>();
        for idx in 0..10 {
            info!("{}", array[idx]);
        }
    }

    sess.close().map_err(|e| Error::Vaccel(e))
}
