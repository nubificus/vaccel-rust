use crate::ffi;
use crate::VaccelId;
use crate::{Error, Result};
use std::any::Any;
use std::ffi::CString;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct SharedObject {
    inner: *mut ffi::vaccel_shared_object,
}

impl SharedObject {
    /// Create a new Saved Model object
    pub fn new(path: &Path) -> Result<Self> {
        let mut shared_obj = Box::<ffi::vaccel_shared_object>::default();

        // We create a CString to ensure that the path we pass to libvaccel
        // is null terminated
        let c_str = CString::new(path.as_os_str().to_str().ok_or(Error::InvalidArgument)?)
            .map_err(|_| Error::InvalidArgument)?;

        match unsafe { ffi::vaccel_shared_object_new(&mut *shared_obj, c_str.as_ptr()) as u32 } {
            ffi::VACCEL_OK => Ok(SharedObject {
                inner: Box::into_raw(shared_obj),
            }),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create a new SharedObject from a vaccel saved model type
    pub fn from_vaccel(inner: *mut ffi::vaccel_shared_object) -> Self {
        SharedObject { inner }
    }

    /// Get the id of the model
    pub fn id(&self) -> VaccelId {
        let inner = unsafe { ffi::vaccel_shared_object_get_id(self.inner) };
        VaccelId::from(inner)
    }

    /// Returns `true` if the model has been initialized
    pub fn initialized(&self) -> bool {
        self.id().has_id()
    }

    pub fn destroy(&mut self) -> Result<()> {
        if !self.initialized() {
            return Ok(());
        }

        match unsafe { ffi::vaccel_shared_object_destroy(self.inner) as u32 } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Create the resource from in-memory data
    pub fn from_in_memory(data: &[u8]) -> Result<Self> {
        let mut shared_obj = Box::<ffi::vaccel_shared_object>::default();

        match unsafe {
            ffi::vaccel_shared_object_new_from_buffer(&mut *shared_obj, data.as_ptr(), data.len())
                as u32
        } {
            ffi::VACCEL_OK => Ok(SharedObject {
                inner: Box::into_raw(shared_obj),
            }),
            err => Err(Error::Runtime(err)),
        }
    }

    /// Return bytes
    pub fn get_bytes(&self) -> Option<&[u8]> {
        let mut size = Default::default();
        let ptr = unsafe { ffi::vaccel_shared_object_get(self.inner, &mut size) };

        if !ptr.is_null() {
            Some(unsafe { std::slice::from_raw_parts(ptr, size) })
        } else {
            None
        }
    }

    pub(crate) fn inner(&self) -> *const ffi::vaccel_shared_object {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> *mut ffi::vaccel_shared_object {
        self.inner
    }
}

impl crate::resource::Resource for SharedObject {
    fn id(&self) -> VaccelId {
        self.id()
    }

    fn initialized(&self) -> bool {
        self.initialized()
    }

    fn to_vaccel_ptr(&self) -> Option<*const ffi::vaccel_resource> {
        if !self.initialized() {
            None
        } else {
            let resource = unsafe { (*self.inner).resource };
            Some(resource)
        }
    }

    fn to_mut_vaccel_ptr(&self) -> Option<*mut ffi::vaccel_resource> {
        if !self.initialized() {
            None
        } else {
            let resource = unsafe { (*self.inner).resource };
            Some(resource)
        }
    }

    fn destroy(&mut self) -> Result<()> {
        self.destroy()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
