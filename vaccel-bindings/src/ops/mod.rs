use crate::{Result, Session};

pub mod genop;
pub mod image;
pub mod noop;
pub mod tensorflow;
pub mod torch;

pub trait InferenceModel<A, T> {
    type LoadResult;

    /// Load an inference session for model
    fn load(&mut self, sess: &mut Session) -> Result<Self::LoadResult>;

    /// Run inference on model
    fn run(&mut self, sess: &mut Session, args: &mut A) -> Result<T>;

    /// Unload an inference session for model
    fn unload(&mut self, sess: &mut Session) -> Result<()>;
}
