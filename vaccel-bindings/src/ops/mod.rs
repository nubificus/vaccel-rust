// SPDX-License-Identifier: Apache-2.0

use crate::{Resource, Result, Session};
use std::pin::Pin;

pub mod genop;
pub mod image;
pub mod noop;
pub mod tensorflow;
pub mod torch;

pub trait ModelInitialize<'a> {
    /// Initialize a Resource of type Model
    fn new(inner: Pin<&'a mut Resource>) -> Pin<Box<Self>>;
}

pub trait ModelRun<'a>: ModelInitialize<'a> {
    type RunArgs;
    type RunResult;

    /// Run inference on model
    fn run(
        self: Pin<&mut Self>,
        sess: &mut Session,
        args: &mut Self::RunArgs,
    ) -> Result<Self::RunResult>;
    fn inner_mut(self: Pin<&mut Self>) -> Pin<&mut Resource>;
}

pub trait ModelLoadUnload<'a>: ModelRun<'a> {
    type LoadResult;

    /// Load an inference session for model
    fn load(self: Pin<&mut Self>, sess: &mut Session) -> Result<Self::LoadResult>;
    /// Unload an inference session for model
    fn unload(self: Pin<&mut Self>, sess: &mut Session) -> Result<()>;
}
