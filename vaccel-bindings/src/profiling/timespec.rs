// SPDX-License-Identifier: Apache-2.0

use super::NSEC_PER_SEC;
use std::time::Duration;

/// Represents a `struct timespec` C object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timespec {
    tv_sec: u64,
    tv_nsec: u32,
}

impl Timespec {
    /// Creates a new zero-initialized `Timespec`.
    pub const fn zero() -> Self {
        Self::new(0, 0)
    }

    /// Creates a new `Timespec`.
    const fn new(tv_sec: u64, tv_nsec: u32) -> Self {
        assert!(tv_nsec < NSEC_PER_SEC);
        Self { tv_sec, tv_nsec }
    }

    /// Creates a new `Timespec` by retrieving the time of the monotonic clock.
    pub fn now() -> Self {
        let mut t = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        let ret = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut t) };
        assert_eq!(ret, 0, "Failed to get monotonic time");

        Self::new(t.tv_sec as u64, t.tv_nsec as u32)
    }

    /// Returns the elapsed time since the `Timespec` timestamp.
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        now.duration_since(*self)
    }

    /// Returns the duration since another `Timespec`.
    pub fn duration_since(&self, earlier: Self) -> Duration {
        let self_duration = Duration::new(self.tv_sec, self.tv_nsec);
        let earlier_duration = Duration::new(earlier.tv_sec, earlier.tv_nsec);
        self_duration - earlier_duration
    }

    /// Returns the `Timespec` as a timestamp in nanoseconds.
    pub const fn as_nanos(&self) -> u128 {
        Duration::new(self.tv_sec, self.tv_nsec).as_nanos()
    }

    /// Creates a `Timespec` from a timestamp in nanoseconds.
    pub const fn from_nanos(nanos: u64) -> Self {
        Self::new(
            nanos / (NSEC_PER_SEC as u64),
            (nanos % (NSEC_PER_SEC as u64)) as u32,
        )
    }
}

impl From<Duration> for Timespec {
    fn from(duration: Duration) -> Self {
        Self::new(duration.as_secs(), duration.subsec_nanos())
    }
}

impl From<Timespec> for Duration {
    fn from(timespec: Timespec) -> Self {
        Duration::new(timespec.tv_sec, timespec.tv_nsec)
    }
}
