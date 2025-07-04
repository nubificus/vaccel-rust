// SPDX-License-Identifier: Apache-2.0

use super::{is_profiling_enabled, Region, RegionStats, Sample};
use crate::ffi;
use std::{
    collections::{btree_map, BTreeMap},
    ops::Deref,
};

/// A collection of profiling regions for a component.
#[derive(Debug, Clone)]
pub struct Profiler {
    regions: BTreeMap<String, Region>,
    component_name: String,
}

impl Profiler {
    /// Creates a new `Profiler` with the given component name.
    pub fn new(component_name: impl Into<String>) -> Self {
        Self {
            regions: BTreeMap::new(),
            component_name: component_name.into(),
        }
    }

    /// Returns the component name of the `Profiler`.
    pub fn component_name(&self) -> &str {
        &self.component_name
    }

    /// Returns the number of profiling regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Returns `true` if there are no profiling regions.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Clears all profiling data.
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Gets the full region name (including the component name prefix).
    fn full_region_name(&self, region_name: &str) -> String {
        format!("[{}] {}", self.component_name, region_name)
    }

    /// Starts profiling for the given region.
    ///
    /// Creates a new sample and adds it to the region's sample collection.
    pub fn start(&mut self, region_name: &str) {
        if !is_profiling_enabled() {
            return;
        }

        let full_name = self.full_region_name(region_name);
        self.regions.entry(full_name).or_default().start_sample();
    }

    /// Stops profiling for the given region.
    ///
    /// Completes the current active sample if one exists.
    pub fn stop(&mut self, region_name: &str) {
        if !is_profiling_enabled() {
            return;
        }

        let full_name = self.full_region_name(region_name);
        if let Some(region) = self.regions.get_mut(&full_name) {
            region.stop_sample();
        }
    }

    /// Returns a region by name (applies component prefix).
    pub fn get(&self, region_name: &str) -> Option<&Region> {
        let full_name = self.full_region_name(region_name);
        self.regions.get(&full_name)
    }

    /// Returns a region by full name (no prefix applied).
    pub fn get_by_full_name(&self, full_region_name: &str) -> Option<&Region> {
        self.regions.get(full_region_name)
    }

    /// Inserts pre-completed samples for a region.
    pub fn insert_samples(&mut self, region_name: &str, samples: Vec<Sample>) {
        if !is_profiling_enabled() {
            return;
        }

        let full_name = self.full_region_name(region_name);
        let region = self.regions.entry(full_name).or_default();
        region.insert_samples(samples);
    }

    /// Returns all regions and their samples for FFI conversion.
    pub fn to_ffi(&self) -> Option<BTreeMap<String, Vec<ffi::vaccel_prof_sample>>> {
        if !is_profiling_enabled() {
            return None;
        }

        Some(
            self.regions
                .iter()
                .map(|(name, region)| {
                    let ffi_samples: Vec<ffi::vaccel_prof_sample> =
                        region.samples().iter().map(|s| s.as_ffi()).collect();
                    (name.clone(), ffi_samples)
                })
                .collect(),
        )
    }

    /// Formats timing information for display.
    fn format_region_timing(name: &str, stats: &RegionStats) -> String {
        format!("{}: {}", name, stats)
    }

    /// Returns timing information for all regions as a string (last sample only).
    pub fn format_all_last(&self) -> String {
        if !is_profiling_enabled() {
            return String::new();
        }

        self.regions
            .iter()
            .filter_map(|(name, region)| {
                region.last_sample().map(|sample| {
                    let stats = RegionStats {
                        total_time: sample.duration(),
                        count: 1,
                        avg_time: sample.duration(),
                        min_time: sample.duration(),
                        max_time: sample.duration(),
                    };
                    Self::format_region_timing(name, &stats)
                })
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns total timing information for all regions as a string.
    pub fn format_all_totals(&self) -> String {
        if !is_profiling_enabled() {
            return String::new();
        }

        self.regions
            .iter()
            .map(|(name, region)| {
                let stats = region.stats();
                Self::format_region_timing(name, &stats)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// This will in turn implement the Iterator trait
impl Deref for Profiler {
    type Target = BTreeMap<String, Region>;

    fn deref(&self) -> &Self::Target {
        &self.regions
    }
}

impl IntoIterator for Profiler {
    type Item = (String, Region);
    type IntoIter = btree_map::IntoIter<String, Region>;

    fn into_iter(self) -> Self::IntoIter {
        self.regions.into_iter()
    }
}

impl Extend<(String, Region)> for Profiler {
    fn extend<T: IntoIterator<Item = (String, Region)>>(&mut self, iter: T) {
        self.regions.extend(iter)
    }
}

/// RAII guard for automatic profiling region management.
///
/// Ensures profiling is properly stopped even if an early return or panic
/// occurs.
pub struct ProfilerScope<'a> {
    profiler: &'a mut Profiler,
    region_name: String,
    active: bool,
}

impl<'a> ProfilerScope<'a> {
    /// Creates a new profiling scope that automatically starts profiling.
    pub fn new(profiler: &'a mut Profiler, region_name: impl Into<String>) -> Self {
        let region_name = region_name.into();
        let active = is_profiling_enabled();

        if active {
            profiler.start(&region_name);
        }

        Self {
            profiler,
            region_name,
            active,
        }
    }

    /// Manually stops profiling before the scope ends.
    pub fn stop(mut self) {
        if self.active {
            self.profiler.stop(&self.region_name);
            self.active = false;
        }
    }
}

impl<'a> Drop for ProfilerScope<'a> {
    fn drop(&mut self) {
        if self.active {
            self.profiler.stop(&self.region_name);
        }
    }
}

/// Convenience macro for profiling.
///
/// Usage: `profile!(profiler, "region_name");`
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr) => {
        let _scope = $crate::ProfilerScope::new($profiler, $name);
    };
}
