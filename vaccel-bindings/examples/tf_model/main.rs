use env_logger::Env;
use log::info;

use vaccel_bindings::vaccel_session;

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
    if let Err(_) = register_model(&mut sess, &mut model) {
        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");
    }

    // Read protobuf data into Vector from File
    let model_data = match vec_from_file(path) {
        Ok(data) => data,
        Err(err) => {
            info!("Destroying session {}", sess.id());
            sess.close()
                .expect("Could not destroy session during cleanup");
            return Err(err);
        }
    };

    // Create a TensorFlow model resource from data
    let mut model2 = match create_from_data(model_data.as_slice()) {
        Ok(model) => model,
        Err(err) => {
            sess.close().unwrap();
            return Err(err);
        }
    };

    // Register model with session
    if let Err(err) = register_model(&mut sess, &mut model2) {
        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");
        return Err(err);
    }

    // Unregister models from session
    if let Err(err) = unregister_model(&mut sess, &mut model) {
        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");
        return Err(err);
    }

    if let Err(err) = unregister_model(&mut sess, &mut model2) {
        info!("Destroying session {}", sess.id());
        sess.close()
            .expect("Could not destroy session during cleanup");
        return Err(err);
    }

    info!("Closing session {}", sess.id());
    sess.close().map_err(|e| Error::Vaccel(e))
}
