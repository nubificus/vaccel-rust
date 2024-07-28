mod utilities;

use env_logger::Env;
use log::{error, info};
use std::path::PathBuf;
use vaccel::{
    ops::{
        tensorflow::lite::{InferenceArgs, InferenceResult, Tensor},
        InferenceModel,
    },
    resources::SingleModel,
    Session,
};

fn main() -> utilities::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut sess = Session::new(0)?;
    info!("New session {}", sess.id());

    let path = PathBuf::from("./examples/files/tf/lstm2.tflite");
    let mut model = SingleModel::new().from_export_dir(&path)?;
    info!("New single model from export dir: {}", model.id());

    // Register model with session
    sess.register(&mut model)?;
    info!("Registered model {} with session {}", model.id(), sess.id());

    // Load model file
    if let Err(e) =
        <SingleModel as InferenceModel<InferenceArgs, InferenceResult>>::load(&mut model, &mut sess)
    {
        error!("Could not load file for model {}: {}", model.id(), e);

        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");

        return Err(utilities::Error::Vaccel(e));
    }

    // Prepare data for inference
    let in_tensor = Tensor::<f32>::new(&[1, 30]).with_data(&[1.0; 30])?;

    let mut sess_args = InferenceArgs::new();
    sess_args.add_input(&in_tensor);
    sess_args.set_nr_outputs(1);

    let result = model.run(&mut sess, &mut sess_args)?;

    match result.get_output::<f32>(0) {
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
        Err(err) => println!("Inference failed: '{}'", err),
    }

    <SingleModel as InferenceModel<InferenceArgs, InferenceResult>>::unload(&mut model, &mut sess)?;

    sess.close()?;

    Ok(())
}
