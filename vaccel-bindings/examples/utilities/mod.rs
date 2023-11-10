#![allow(dead_code)]

use std::fmt;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use vaccel::Error as VaccelError;

pub enum Error {
    IO(std::io::Error),
    Vaccel(VaccelError),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => write!(f, "{}", err),
            Error::Vaccel(err) => write!(f, "vAccel runtime error: {}", err),
        }
    }
}

impl From<VaccelError> for Error {
    fn from(error: VaccelError) -> Self {
        Error::Vaccel(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn vec_from_file(path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(path).map_err(Error::IO)?;

    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(Error::IO)?;

    Ok(data)
}

pub fn load_in_mem(path: &Path) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let mut model_path = PathBuf::from(path);
    model_path.push("saved_model.pb");
    let model_pb = vec_from_file(&model_path)?;

    let mut ckpt_path = PathBuf::from(path);
    ckpt_path.push("variables/variables.data-00000-of-00001");
    let checkpoint = vec_from_file(&ckpt_path)?;

    let mut var_index_path = PathBuf::from(path);
    var_index_path.push("variables/variables.index");
    let var_index = vec_from_file(&var_index_path)?;

    Ok((model_pb, checkpoint, var_index))
}
