//! context for global activity tracker

use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct ActivityTracker {
    /// Map of activity IDs to their counts
    inner: RwSignal<HashMap<Uuid, u32>>,
    /// Signal indicating if any activity is active
    pub is_active: Signal<bool>,
}

impl ActivityTracker {
    /// Create a new ActivityTracker
    pub fn new() -> Self {
        let inner = RwSignal::new(HashMap::new());
        let is_active = Signal::derive(move || {
            inner.with(|activity_map| activity_map.values().any(|v| *v > 0))
        });
        Self { inner, is_active }
    }

    /// Track pending memo for component
    pub fn track_pending_memo(&self, component_id: Uuid, pending: Memo<bool>) {
        let inner = self.inner;
        Effect::new(move || {
            if let Some(pending) = pending.try_get() {
                inner.update(|activity_map| {
                    let count = activity_map.entry(component_id).or_insert(0);
                    if pending {
                        *count += 1;
                    } else if *count > 0 {
                        *count -= 1;
                    }
                });
            }
        });
    }

    /// remove component from activity tracker
    pub fn remove_component(&self, component_id: Uuid) {
        self.inner.update(|activity_map| {
            activity_map.remove(&component_id);
        });
    }

    pub async fn track_activity_wrapper<F, Fut>(
        &self,
        component_id: Uuid,
        activity_future: F,
    ) -> Fut::Output
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future,
    {
        // Increment activity count for the component
        self.inner.update(|activity_map| {
            let count = activity_map.entry(component_id).or_insert(0);
            *count += 1;
        });

        // Await the provided future
        let result = activity_future().await;

        // Decrement activity count for the component
        self.inner.update(|activity_map| {
            if let Some(count) = activity_map.get_mut(&component_id) {
                *count = count.saturating_sub(1);
            }
        });

        result
    }
}
