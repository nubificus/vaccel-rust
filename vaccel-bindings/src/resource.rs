use crate::ffi;
use crate::Result;
use crate::VaccelId;
use std::any::Any;

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
