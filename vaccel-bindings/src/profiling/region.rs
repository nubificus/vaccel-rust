// SPDX-License-Identifier: Apache-2.0

use super::sample::{ActiveSample, Sample};
use derive_more::Display;
use std::time::Duration;

/// Statistics for a collection of samples in a profiling region.
#[derive(Debug, Default, Display, Clone, Copy)]
#[display("total_time: {} nsec nr_entries: {}", self.total_time.as_nanos(), self.count)]
pub struct RegionStats {
    pub total_time: Duration,
    pub count: usize,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
}

impl RegionStats {
    pub fn from_samples(samples: &[Sample]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let durations: Vec<Duration> = samples.iter().map(|s| s.duration()).collect();
        let total_time = durations.iter().sum();
        let count = samples.len();
        let avg_time = total_time / count as u32;
        let min_time = durations.iter().min().copied().unwrap_or_default();
        let max_time = durations.iter().max().copied().unwrap_or_default();

        Self {
            total_time,
            count,
            avg_time,
            min_time,
            max_time,
        }
    }
}

/// Data for a single profiling region.
#[derive(Debug, Default, Clone)]
pub struct Region {
    samples: Vec<Sample>,
    active_sample: Option<ActiveSample>,
}

impl Region {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            active_sample: None,
        }
    }

    /// Starts a new sample for this region.
    ///
    /// If there's already an active sample, it will be completed first.
    pub fn start_sample(&mut self) {
        // Complete any existing active sample
        if let Some(active) = self.active_sample.take() {
            self.samples.push(active.finish());
        }

        // Start a new sample
        self.active_sample = Some(ActiveSample::start_now());
    }

    /// Completes the currently active sample.
    pub fn stop_sample(&mut self) {
        if let Some(active) = self.active_sample.take() {
            self.samples.push(active.finish());
        }
    }

    /// Returns all completed samples.
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    /// Returns the currently active sample, if any.
    pub fn active_sample(&self) -> Option<&ActiveSample> {
        self.active_sample.as_ref()
    }

    /// Returns statistics for this region.
    pub fn stats(&self) -> RegionStats {
        RegionStats::from_samples(&self.samples)
    }

    /// Returns the last completed sample.
    pub fn last_sample(&self) -> Option<&Sample> {
        self.samples.last()
    }

    /// Inserts pre-completed samples to the region samples.
    pub fn insert_samples(&mut self, samples: Vec<Sample>) {
        self.samples.extend(samples);
    }
}
