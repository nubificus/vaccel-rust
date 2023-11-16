use crate::{ffi, Session, Error, Result, profiling::ProfRegions};

use protocols::genop::GenopArg as ProtGenopArg;

#[derive(Debug)]
pub struct GenopArg {
    inner: ffi::vaccel_arg,
    buf: Vec<u8>,
    size: usize,
    argtype: usize,
}

impl GenopArg {
    pub fn new(buffer: &mut [u8], size: usize, argtype: usize) -> Self {
        let mut b = buffer.to_owned();
        GenopArg {
            inner: ffi::vaccel_arg {
                buf: b.as_mut_ptr() as *mut libc::c_void,
                size: size as u32,
                argtype: argtype as u32,
            },
            buf: b,
            size: size,
            argtype: argtype,
        }
    }
    pub fn get_size(&self) -> u32 {
        self.inner.size
    }

    pub fn set_size(&mut self, v: usize) {
        self.size = v;
        self.inner.size = v as u32;
    }

    pub fn get_buf(&self) -> *mut u8 {
        self.inner.buf as *mut u8
    }

    pub fn get_argtype(&self) -> u32 {
        self.inner.argtype
    }

    pub fn set_buf(&mut self, b: &mut [u8]) {
        self.buf = b.to_owned();
    }
}

impl From<&mut ProtGenopArg> for GenopArg {
    fn from(arg: &mut ProtGenopArg) -> Self {
        let argtype = arg.get_argtype();
        let size = arg.get_size();
        let buf = arg.mut_buf();
        GenopArg::new(buf, size as usize, argtype as usize)
    }
}

impl From<&GenopArg> for ProtGenopArg {
    fn from(arg: &GenopArg) -> Self {
        ProtGenopArg {
            buf: arg.buf.to_owned(),
            size: arg.size as u32,
            ..Default::default()
        }
    }
}

impl Session {
    /// vAccel generic operation
    ///
    /// Execute an arbitrary vAccel operation passing to vaccelrt arguments
    /// in the generic form of `vaccel_arg` slices. `vaccel_arg` describes an
    /// argument as a generic `void *` pointer and a size.
    ///
    /// # Arguments
    ///
    /// * `read` - A slice of `vaccel_arg` with the arguments that are read only. The first
    /// argument of the slice is the type of the operation
    /// * `write` - A slice of `vaccel_arg` with the read-write arguments of the operation.
    pub fn genop(&mut self, read: &mut [GenopArg], write: &mut [GenopArg], timers: &mut ProfRegions) -> Result<()> {
        timers.start("genop > session > read_args");
        let mut read_args: Vec<ffi::vaccel_arg> = read.iter().map(|e| e.inner).collect();
        let mut write_args: Vec<ffi::vaccel_arg> = write.iter().map(|e| e.inner).collect();
        timers.stop("genop > session > read_args");

        match unsafe {
            timers.start("genop > session > vaccel_genop");
            let res = ffi::vaccel_genop(
                self.inner_mut(),
                read_args.as_mut_ptr(),
                read_args.len() as i32,
                write_args.as_mut_ptr(),
                write_args.len() as i32,
            );
            timers.stop("genop > session > vaccel_genop");
            res as u32
        } {
            ffi::VACCEL_OK => Ok(()),
            err => Err(Error::Runtime(err)),
        }
    }
}
