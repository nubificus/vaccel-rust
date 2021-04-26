use crate::Result;
use crate::{vaccel_id_t, vaccel_resource};

pub trait VaccelResource {
    /// Get the id of a vAccel resource
    fn id(&self) -> Option<vaccel_id_t>;

    /// Has the vAccel resource been created?
    fn initialized(&self) -> bool;

    /// Get a const pointer of the underlying vAccel resource
    fn to_vaccel_ptr(&self) -> Option<*const vaccel_resource>;

    /// Get a mutable pointer of the underlying vAccel resource
    fn to_mut_vaccel_ptr(&self) -> Option<*mut vaccel_resource>;

    /// Destroy a resource
    fn destroy(&mut self) -> Result<()>;
}
