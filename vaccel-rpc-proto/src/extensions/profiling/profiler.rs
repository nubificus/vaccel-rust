// SPDX-License-Identifier: Apache-2.0

use crate::profiling::{Profiler, Region};
use std::collections::HashMap;
use vaccel::profiling::{Profiler as VaccelProfiler, Region as VaccelRegion};

impl From<&Profiler> for VaccelProfiler {
    fn from(profiler: &Profiler) -> Self {
        let vaccel_regions = profiler.regions.iter().map(|(region_name, region)| {
            let vaccel_region: VaccelRegion = region.into();
            (region_name.to_string(), vaccel_region)
        });

        let mut vaccel_profiler = VaccelProfiler::new(profiler.component_name.clone());
        vaccel_profiler.extend(vaccel_regions);
        vaccel_profiler
    }
}

impl From<Profiler> for VaccelProfiler {
    fn from(profiler: Profiler) -> Self {
        let vaccel_regions = profiler.regions.into_iter().map(|(region_name, region)| {
            let vaccel_region: VaccelRegion = region.into();
            (region_name, vaccel_region)
        });

        let mut vaccel_profiler = VaccelProfiler::new(profiler.component_name);
        vaccel_profiler.extend(vaccel_regions);
        vaccel_profiler
    }
}

impl From<&VaccelProfiler> for Profiler {
    fn from(vaccel: &VaccelProfiler) -> Self {
        let component_name = vaccel.component_name().to_string();
        let regions: HashMap<String, Region> = vaccel
            .iter()
            .map(|(region_name, region)| (region_name.to_string(), region.into()))
            .collect();

        Self {
            component_name,
            regions,
            ..Default::default()
        }
    }
}

impl From<VaccelProfiler> for Profiler {
    fn from(vaccel: VaccelProfiler) -> Self {
        let component_name = vaccel.component_name().to_string();
        let regions: HashMap<String, Region> = vaccel
            .into_iter()
            .map(|(region_name, region)| (region_name, region.into()))
            .collect();

        Self {
            component_name,
            regions,
            ..Default::default()
        }
    }
}
