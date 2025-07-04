// SPDX-License-Identifier: Apache-2.0

use crate::profiling::Region;
use vaccel::profiling::{Region as VaccelRegion, Sample as VaccelSample};

impl From<&Region> for VaccelRegion {
    fn from(region: &Region) -> Self {
        let vaccel_samples: Vec<VaccelSample> = region.samples.iter().map(|s| s.into()).collect();

        let mut vaccel_region = VaccelRegion::new();
        vaccel_region.insert_samples(vaccel_samples);
        vaccel_region
    }
}

impl From<Region> for VaccelRegion {
    fn from(region: Region) -> Self {
        let vaccel_samples: Vec<VaccelSample> =
            region.samples.into_iter().map(|s| s.into()).collect();

        let mut vaccel_region = VaccelRegion::new();
        vaccel_region.insert_samples(vaccel_samples);
        vaccel_region
    }
}

impl From<&VaccelRegion> for Region {
    fn from(vaccel: &VaccelRegion) -> Self {
        Self {
            samples: vaccel.samples().iter().map(|s| (*s).into()).collect(),
            ..Default::default()
        }
    }
}

impl From<VaccelRegion> for Region {
    fn from(vaccel: VaccelRegion) -> Self {
        Self::from(&vaccel)
    }
}
