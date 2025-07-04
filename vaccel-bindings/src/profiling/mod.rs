// SPDX-License-Identifier: Apache-2.0

use crate::ffi;

pub mod profiler;
pub mod profiler_manager;
pub mod region;
pub mod sample;
pub mod timespec;

pub use profiler::{Profiler, ProfilerScope};
pub use profiler_manager::{ProfilerManager, ProfilerManagerScope, SessionProfiler};
pub use region::{Region, RegionStats};
pub use sample::Sample;
pub use timespec::Timespec;

const NSEC_PER_SEC: u32 = 1_000_000_000;

/// Checks if profiling is enabled at both compile-time and runtime.
#[inline]
fn is_profiling_enabled() -> bool {
    cfg!(feature = "profiling") && unsafe { ffi::vaccel_prof_enabled() }
}
