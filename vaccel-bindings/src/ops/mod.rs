// SPDX-License-Identifier: Apache-2.0

use crate::{Resource, Result, Session};

pub mod genop;
pub mod image;
pub mod noop;
pub mod tensorflow;
pub mod torch;

pub trait ModelInitialize<'a> {
    /// Initializes a `Resource` of model type
    fn new(inner: &'a mut Resource) -> Self;
}

pub trait ModelRun<'a>: ModelInitialize<'a> {
    type RunArgs;
    type RunResult;

    /// Runs inference on model
    fn run(&mut self, sess: &mut Session, args: &mut Self::RunArgs) -> Result<Self::RunResult>;
}

pub trait ModelLoadUnload<'a>: ModelRun<'a> {
    type LoadUnloadResult;

    /// Loads an inference session for model
    fn load(&mut self, sess: &mut Session) -> Result<Self::LoadUnloadResult>;

    /// Unloads an inference session for model
    fn unload(&mut self, sess: &mut Session) -> Result<Self::LoadUnloadResult>;
}
