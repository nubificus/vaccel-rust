// SPDX-License-Identifier: Apache-2.0

use crate::{Result, Session};
use std::pin::Pin;

pub mod genop;
pub mod image;
pub mod noop;
pub mod tensorflow;
pub mod torch;

pub trait InferenceModel<A, T> {
    type LoadResult;

    /// Load an inference session for model
    fn load(self: Pin<&mut Self>, sess: &mut Session) -> Result<Self::LoadResult>;

    /// Run inference on model
    fn run(self: Pin<&mut Self>, sess: &mut Session, args: &mut A) -> Result<T>;

    /// Unload an inference session for model
    fn unload(self: Pin<&mut Self>, sess: &mut Session) -> Result<()>;
}
