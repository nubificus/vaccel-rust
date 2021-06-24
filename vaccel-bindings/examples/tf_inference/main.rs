use env_logger::Env;
use log::{error, info};

use std::path::PathBuf;

use vaccel::ops::inference;
use vaccel::tensorflow as tf;
use vaccel::Session;

extern crate utilities;

fn main() -> utilities::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut sess = Session::new(0)?;
    info!("New session {}", sess.id());

    let path = PathBuf::from("./examples/files/tf/lstm2");
    let mut model = tf::SavedModel::new().from_export_dir(&path)?;
    info!("New saved model from export dir: {}", model.id());

    // Register model with session
    sess.register(&mut model)?;
    info!("Registered model {} with session {}", model.id(), sess.id());

    // Load model graph
    if let Err(err) = model.load_graph(&mut sess) {
        error!("Could not load graph for model {}: {}", model.id(), err);

        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");

        return Err(utilities::Error::Vaccel(err));
    }

    // Prepare data for inference
    let run_options = tf::Buffer::new(&[]);
    let in_tensor = tf::Tensor::<f32>::new(&[1, 30]).with_data(&[1.0; 30])?;
    let in_node = tf::Node::new("serving_default_input_1", 0);
    let out_node = tf::Node::new("StatefulPartitionedCall", 0);

    let mut sess_args = inference::InferenceArgs::new();
    sess_args.set_run_options(&run_options);
    sess_args.add_input(&in_node, &in_tensor);
    sess_args.request_output(&out_node);

    let result = model.inference(&mut sess, &mut sess_args)?;

    let out = result.get_output::<f32>(0).unwrap();

    println!("Success!");
    println!(
        "Output tensor => type:{:?} nr_dims:{}",
        out.data_type(),
        out.nr_dims()
    );
    for i in 0..out.nr_dims() {
        println!("dim[{}]: {}", i, out.dim(i as usize).unwrap());
    }
    println!("Result Tensor :");
    for i in 0..10 {
        println!("{:.6}", out[i]);
    }

    sess.close()?;

    Ok(())
}
