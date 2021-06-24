use env_logger::Env;
use log::info;

use vaccel::tensorflow::SavedModel;
use vaccel::Session;

use std::path::PathBuf;

extern crate utilities;
use utilities::*;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let mut sess = Session::new(0)?;
    info!("New session {}", sess.id());

    let path = PathBuf::from("./examples/files/tf/lstm2");
    let mut model = SavedModel::new().from_export_dir(&path)?;
    info!("New saved model from export dir: {}", model.id());

    // Register model with session
    sess.register(&mut model)?;
    info!("Registered model {} with session {}", model.id(), sess.id());

    // Read saved model data in memory
    let (model_pb, ckpt, var_index) = utilities::load_in_mem(&path)?;

    // Create a TensorFlow model resource from data
    let mut model2 = SavedModel::new().from_in_memory(model_pb, ckpt, var_index)?;
    info!("New saved model from in-memory data: {}", model.id());
    sess.register(&mut model2)?;
    info!(
        "Registered model {} with session {}",
        model2.id(),
        sess.id()
    );

    // Unregister models from session
    sess.unregister(&mut model)?;
    info!(
        "Unregistered model {} from session {}",
        model.id(),
        sess.id()
    );

    sess.unregister(&mut model2)?;
    info!(
        "Unregistered model {} from session {}",
        model2.id(),
        sess.id()
    );

    info!("Closing session {}", sess.id());
    sess.close()?;
    Ok(())
}
