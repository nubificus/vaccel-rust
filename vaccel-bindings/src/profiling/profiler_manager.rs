// SPDX-License-Identifier: Apache-2.0

use super::{is_profiling_enabled, Profiler};
use crate::VaccelId;
use dashmap::DashMap;
use std::sync::Arc;

/// Session-based profiler manager that handles multiple profilers indexed by
/// session ID.
#[derive(Debug, Clone)]
pub struct ProfilerManager {
    profilers: Arc<DashMap<VaccelId, Profiler>>,
    default_name: String,
}

impl ProfilerManager {
    /// Creates a new `ProfilerManager`.
    pub fn new(default_name: impl Into<String>) -> Self {
        Self {
            profilers: Arc::new(DashMap::new()),
            default_name: default_name.into(),
        }
    }

    /// Starts profiling for a region in the given session.
    pub fn start(&self, session_id: VaccelId, region_name: &str) {
        if !is_profiling_enabled() {
            return;
        }
        self.profilers
            .entry(session_id)
            .or_insert_with(|| Profiler::new(&self.default_name))
            .start(region_name);
    }

    /// Stops profiling for a region in the given session.
    pub fn stop(&self, session_id: VaccelId, region_name: &str) {
        if !is_profiling_enabled() {
            return;
        }
        if let Some(mut profiler) = self.profilers.get_mut(&session_id) {
            profiler.stop(region_name);
        }
    }

    /// Creates a profiler scope for automatic cleanup.
    pub fn scope(&self, session_id: VaccelId, region_name: &str) -> ProfilerManagerScope {
        ProfilerManagerScope::new(self, session_id, region_name)
    }

    /// Profiles a closure automatically.
    pub fn profile_fn<F, R>(&self, session_id: VaccelId, region_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _scope = self.scope(session_id, region_name);
        f()
    }

    /// Profiles an async function automatically.
    ///
    /// Async version of the `profile_fn` method.
    pub async fn profile_async_fn<F, Fut, R>(
        &self,
        session_id: VaccelId,
        region_name: &str,
        f: F,
    ) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let _scope = self.scope(session_id, region_name);
        f().await
    }

    /// Returns a reference to a profiler for a session.
    pub fn get(
        &self,
        session_id: VaccelId,
    ) -> Option<dashmap::mapref::one::Ref<VaccelId, Profiler>> {
        self.profilers.get(&session_id)
    }

    /// Clears profiling data for a session.
    pub fn reset(&self, session_id: VaccelId) {
        if let Some(mut profiler) = self.profilers.get_mut(&session_id) {
            profiler.clear();
        }
    }

    /// Removes and returns the profiler for a session.
    pub fn remove(&self, session_id: VaccelId) -> Option<Profiler> {
        self.profilers
            .remove(&session_id)
            .map(|(_, profiler)| profiler)
    }

    /// Merges the regions of another profiler into a session's profiler.
    pub fn merge_profiler(&self, session_id: VaccelId, other: Profiler) {
        if !is_profiling_enabled() {
            return;
        }

        self.profilers
            .entry(session_id)
            .or_insert_with(|| Profiler::new(&self.default_name))
            .extend(other);
    }
}

/// RAII guard for session-based profiling.
pub struct ProfilerManagerScope<'a> {
    profiler: &'a ProfilerManager,
    session_id: VaccelId,
    region_name: String,
    active: bool,
}

impl<'a> ProfilerManagerScope<'a> {
    /// Creates a new profiling scope that automatically starts profiling.
    fn new(profiler: &'a ProfilerManager, session_id: VaccelId, region_name: &str) -> Self {
        let active = is_profiling_enabled();
        if active {
            profiler.start(session_id, region_name);
        }

        Self {
            profiler,
            session_id,
            region_name: region_name.to_string(),
            active,
        }
    }

    /// Manually stops profiling before the scope ends.
    pub fn stop(mut self) {
        if self.active {
            self.profiler.stop(self.session_id, &self.region_name);
            self.active = false;
        }
    }
}

impl<'a> Drop for ProfilerManagerScope<'a> {
    fn drop(&mut self) {
        if self.active {
            self.profiler.stop(self.session_id, &self.region_name);
        }
    }
}

/// Convenience macro for session-based profiling.
#[macro_export]
macro_rules! profile_session {
    ($profiler:expr, $session_id:expr, $region_name:expr) => {
        let _session_scope = $profiler.scope($session_id, $region_name);
    };
    ($profiler:expr, $session_id:expr, $region_name:expr, $block:block) => {{
        let _session_scope = $profiler.scope($session_id, $region_name);
        $block
    }};
}

/// Trait for types that can perform session-based profiling operations.
pub trait SessionProfiler {
    /// Starts profiling for a region in the given session.
    fn start_profiling(&self, session_id: VaccelId, region_name: &str);

    /// Stops profiling for a region in a session
    fn stop_profiling(&self, session_id: VaccelId, region_name: &str);

    /// Creates a profiler scope for automatic cleanup.
    fn profile_scope(&self, session_id: VaccelId, region_name: &str) -> ProfilerManagerScope;

    /// Profiles a closure automatically.
    fn profile_fn<F, R>(&self, session_id: VaccelId, region_name: &str, f: F) -> R
    where
        F: FnOnce() -> R;

    /// Profiles an async function automatically.
    fn profile_async_fn<F, Fut, R>(
        &self,
        session_id: VaccelId,
        region_name: &str,
        f: F,
    ) -> impl std::future::Future<Output = R>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>;
}

// Blanket implementation for any type that has a `ProfilerManager`
impl<T> SessionProfiler for T
where
    T: AsRef<ProfilerManager>,
{
    fn start_profiling(&self, session_id: VaccelId, region_name: &str) {
        self.as_ref().start(session_id, region_name)
    }

    fn stop_profiling(&self, session_id: VaccelId, region_name: &str) {
        self.as_ref().stop(session_id, region_name)
    }

    fn profile_scope(&self, session_id: VaccelId, region_name: &str) -> ProfilerManagerScope {
        self.as_ref().scope(session_id, region_name)
    }

    fn profile_fn<F, R>(&self, session_id: VaccelId, region_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.as_ref().profile_fn(session_id, region_name, f)
    }

    fn profile_async_fn<F, Fut, R>(
        &self,
        session_id: VaccelId,
        region_name: &str,
        f: F,
    ) -> impl std::future::Future<Output = R>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        self.as_ref().profile_async_fn(session_id, region_name, f)
    }
}
