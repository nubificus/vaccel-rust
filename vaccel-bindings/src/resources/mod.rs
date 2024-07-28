use crate::{ffi, Result, VaccelId};
use std::any::Any;

pub mod shared_obj;
pub mod single_model;
#[cfg(target_pointer_width = "64")]
pub mod tf_saved_model;

pub use shared_obj::SharedObject;
pub use single_model::SingleModel;
#[cfg(target_pointer_width = "64")]
pub use tf_saved_model::TFSavedModel;

pub trait Resource {
    /// Get the id of a vAccel resource
    fn id(&self) -> VaccelId;

    /// Has the vAccel resource been created?
    fn initialized(&self) -> bool;

    /// Get a const pointer of the underlying vAccel resource
    fn to_vaccel_ptr(&self) -> Option<*const ffi::vaccel_resource>;

    /// Get a mutable pointer of the underlying vAccel resource
    fn to_mut_vaccel_ptr(&self) -> Option<*mut ffi::vaccel_resource>;

    /// Destroy a resource
    fn destroy(&mut self) -> Result<()>;

    /// "Cast" VaccelResource to Any type sto we can downcast to type
    fn as_any(&self) -> &dyn Any;

    /// "Cast" VaccelResource to a mutable Any type
    fn as_mut_any(&mut self) -> &mut dyn Any;
}
