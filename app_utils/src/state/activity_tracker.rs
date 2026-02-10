//! context for global activity tracker

use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct ActivityTracker {
    /// Map of activity IDs to their counts
    activity_map: RwSignal<HashMap<Uuid, u32>>,
    /// Setter for router activity
    pub set_router_activity: SignalSetter<bool>,
    /// Signal indicating if any activity is active
    pub is_active: Signal<bool>,
}

impl ActivityTracker {
    /// Create a new ActivityTracker
    pub fn new() -> Self {
        let activity_map = RwSignal::new(HashMap::new());
        let router_activity = RwSignal::new(false);
        let (get_router_activity, set_router_activity) = create_slice(
            router_activity,
            |router_activity| *router_activity,
            |router_activity, new_value| *router_activity = new_value,
        );
        let is_active = Signal::derive(move || {
            get_router_activity.get()
                || activity_map.with(|activity_map| activity_map.values().any(|v| *v > 0))
        });
        Self {
            activity_map,
            set_router_activity,
            is_active,
        }
    }

    /// Track pending memo for component
    pub fn track_pending_memo(&self, component_id: Uuid, pending: Memo<bool>) {
        let activity_map = self.activity_map;
        Effect::new(move || {
            if let Some(pending) = pending.try_get() {
                activity_map.update(|activity_map| {
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
        self.activity_map.update(|activity_map| {
            activity_map.remove(&component_id);
        });
    }

    /// Track activity for a future
    pub async fn track_activity_wrapper<Fut>(
        &self,
        component_id: Uuid,
        activity_future: Fut,
    ) -> Fut::Output
    where
        Fut: std::future::Future,
    {
        // Increment activity count for the component
        self.activity_map.update(|activity_map| {
            let count = activity_map.entry(component_id).or_insert(0);
            *count += 1;
        });

        // Await the provided future
        let result = activity_future.await;

        // Decrement activity count for the component
        self.activity_map.update(|activity_map| {
            if let Some(count) = activity_map.get_mut(&component_id) {
                *count = count.saturating_sub(1);
            }
        });

        result
    }
}
