mod utilities;

use env_logger::Env;
use log::info;
use std::path::PathBuf;
use vaccel::{
    ops::{torch, torch::InferenceArgs, InferenceModel},
    resources::SingleModel,
    Session,
};

fn main() -> utilities::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let mut sess = Session::new(0)?;
    info!("New session {}", sess.id());

    let path = PathBuf::from("./examples/files/torch/cnn_trace.pt");
    let mut model = SingleModel::new().from_export_dir(&path)?;
    info!("New saved model from export dir: {}", model.id());

    // Register for the model
    sess.register(&mut model)?;
    info!("Registered model {} with session {}", model.id(), sess.id());

    // Prepare data
    let run_options = torch::Buffer::new(&[]); // vaccel torch buffer with data and size
                                               // TODO: in_tensor setting, use random inputs here, but should be images instead
    let in_tensor = torch::Tensor::<f32>::new(&[3 * 224 * 224]).with_data(&[1.0; 3 * 224 * 224])?;
    info!("in_tensor dim: {}", in_tensor.nr_dims());

    let mut sess_args = InferenceArgs::new();

    sess_args.set_run_options(&run_options);
    sess_args.add_input(&in_tensor);
    sess_args.set_nr_outputs(1);

    let result = model.run(&mut sess, &mut sess_args)?;

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
        Err(err) => println!("Torch JitLoadForward failed: '{}'", err),
    }

    sess.close()?;
    Ok(())
}
