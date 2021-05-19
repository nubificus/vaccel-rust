use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use log::{error, info};

extern crate vaccel_bindings;
use vaccel_bindings::resource::VaccelResource;
use vaccel_bindings::{vaccel_session, vaccel_tf_model};

pub enum Error {
    IO(std::io::Error),

    Vaccel(vaccel_bindings::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => write!(f, "{}", err),
            Error::Vaccel(err) => write!(f, "vAccel runtime error: {}", err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn create_from_file(path: &Path) -> Result<vaccel_tf_model> {
    match vaccel_tf_model::new(path) {
        Ok(model) => {
            info!("New model from file: {}", model.id());
            Ok(model)
        }
        Err(err) => {
            error!("Could not create model from {:?}: {}", path, err);
            Err(Error::Vaccel(err))
        }
    }
}

pub fn vec_from_file(path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(path).map_err(|e| Error::IO(e))?;

    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| Error::IO(e))?;

    Ok(data)
}

pub fn create_from_data(data: &[u8]) -> Result<vaccel_tf_model> {
    match vaccel_tf_model::from_buffer(data) {
        Ok(model) => {
            info!("New model from buffer: {}", model.id());
            Ok(model)
        }
        Err(err) => {
            error!("Could not create model from buffer: {}", err);
            Err(Error::Vaccel(err))
        }
    }
}

pub fn register_model(sess: &mut vaccel_session, model: &mut vaccel_tf_model) -> Result<()> {
    match sess.register(model) {
        Ok(()) => {
            info!("Model {} registered with session {}", model.id(), sess.id());
            Ok(())
        }
        Err(err) => {
            error!(
                "Could not register model {} with session {}",
                model.id(),
                sess.id()
            );
            Err(Error::Vaccel(err))
        }
    }
}

pub fn unregister_model(sess: &mut vaccel_session, model: &mut vaccel_tf_model) -> Result<()> {
    match sess.unregister(model) {
        Ok(()) => {
            info!(
                "Model {} unregistered from session {}",
                model.id(),
                sess.id()
            );
            Ok(())
        }
        Err(err) => {
            error!(
                "Could not unregister model {} from session {}",
                model.id(),
                sess.id()
            );
            Err(Error::Vaccel(err))
        }
    }
}
