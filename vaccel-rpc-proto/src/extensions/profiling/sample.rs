// SPDX-License-Identifier: Apache-2.0

use crate::profiling::Sample;
use std::time::Duration;
use vaccel::profiling::{Sample as VaccelSample, Timespec};

impl From<&Sample> for VaccelSample {
    fn from(sample: &Sample) -> Self {
        Self::new(
            Timespec::from_nanos(sample.start),
            Duration::from_nanos(sample.time),
        )
    }
}

impl From<Sample> for VaccelSample {
    fn from(sample: Sample) -> Self {
        Self::from(&sample)
    }
}

impl From<&VaccelSample> for Sample {
    fn from(vaccel: &VaccelSample) -> Self {
        Self {
            start: vaccel.start_time().as_nanos() as u64,
            time: vaccel.duration().as_nanos() as u64,
            ..Default::default()
        }
    }
}

impl From<VaccelSample> for Sample {
    fn from(vaccel: VaccelSample) -> Self {
        Self::from(&vaccel)
    }
}
