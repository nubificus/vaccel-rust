// SPDX-License-Identifier: Apache-2.0

use super::timespec::Timespec;
use crate::ffi;
use std::time::Duration;
use vaccel_rpc_proto::profiling::Sample as ProtoSample;

/// Represents a completed profiling sample.
///
/// Wrapper for the `struct vaccel_prof_sample` C object.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    inner: ffi::vaccel_prof_sample,
}

impl Sample {
    /// Creates a new completed sample.
    pub fn new(start: Timespec, duration: Duration) -> Self {
        let start_nanos = start.as_nanos();
        let duration_nanos = duration.as_nanos();

        assert!(
            start_nanos <= u64::MAX as u128,
            "Start time too large for FFI"
        );
        assert!(
            duration_nanos <= u64::MAX as u128,
            "Duration too large for FFI"
        );

        Self {
            inner: ffi::vaccel_prof_sample {
                start: start_nanos as u64,
                time: duration_nanos as u64,
            },
        }
    }

    /// Returns the start time of this sample.
    pub fn start_time(&self) -> Timespec {
        Timespec::from_nanos(self.inner.start)
    }

    /// Returns the duration of this sample.
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(self.inner.time)
    }

    /// Returns the FFI representation of this sample.
    pub fn as_ffi(&self) -> ffi::vaccel_prof_sample {
        self.inner
    }
}

/// Represents an active profiling sample that hasn't been completed yet.
#[derive(Debug, Clone, Copy)]
pub struct ActiveSample {
    start: Timespec,
}

impl ActiveSample {
    /// Creates a new active sample starting now.
    pub fn start_now() -> Self {
        Self {
            start: Timespec::now(),
        }
    }

    /// Completes the sample, returning a finished `Sample`.
    pub fn finish(self) -> Sample {
        let duration = self.start.elapsed();
        Sample::new(self.start, duration)
    }

    /// Returns the start time of the active sample.
    pub fn start_time(&self) -> Timespec {
        self.start
    }

    /// Returns the current elapsed time (without finishing the sample).
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl From<&ProtoSample> for Sample {
    fn from(proto: &ProtoSample) -> Self {
        Self::new(
            Timespec::from_nanos(proto.start),
            Duration::from_nanos(proto.time),
        )
    }
}

impl From<ProtoSample> for Sample {
    fn from(proto: ProtoSample) -> Self {
        Self::from(&proto)
    }
}

impl From<&Sample> for ProtoSample {
    fn from(sample: &Sample) -> Self {
        Self {
            start: sample.start_time().as_nanos() as u64,
            time: sample.duration().as_nanos() as u64,
            ..Default::default()
        }
    }
}

impl From<Sample> for ProtoSample {
    fn from(sample: Sample) -> Self {
        Self::from(&sample)
    }
}
