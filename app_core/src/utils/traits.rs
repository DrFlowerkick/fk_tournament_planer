//! traits for utils

use crate::utils::id_version::IdVersion;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// --- Traits for Object Identification ---

pub trait ObjectIdVersion {
    fn get_id_version(&self) -> IdVersion;
}

pub trait ObjectNumber {
    fn get_object_number(&self) -> u32;
}

// --- Traits for Change Detection ---

pub trait Diffable<T> {
    type Diff;
    /// Optional context to filter what should be diffed (e.g. a HashSet of valid keys)
    type Filter: ?Sized;

    /// Compare self with origin and return changes (updates/inserts).
    /// Can optionally be filtered by a context (e.g. valid graph nodes).
    fn get_diff(&self, origin: &Self, filter: Option<&Self::Filter>) -> Self::Diff;
}

// 1. Implementation for Option (Filter is irrelevant here)
impl<T> Diffable<T> for Option<T>
where
    T: PartialEq + Clone,
{
    type Diff = Option<T>;
    type Filter = (); // No filter needed

    fn get_diff(&self, origin: &Self, _filter: Option<&Self::Filter>) -> Self::Diff {
        match (self, origin) {
            (Some(curr), Some(orig)) if curr != orig => Some(curr.clone()), // Modified
            (Some(curr), None) => Some(curr.clone()),                       // New
            _ => None,
        }
    }
}

// 2. Implementation for HashMap (Filter is a HashSet of valid Keys)
impl<T> Diffable<T> for HashMap<Uuid, T>
where
    T: PartialEq + Clone,
{
    type Diff = Vec<T>;
    type Filter = HashSet<Uuid>;

    fn get_diff(&self, origin: &Self, valid_keys: Option<&Self::Filter>) -> Self::Diff {
        // Helper closure to check a single item (avoiding code duplication)
        let check_item = |id: &Uuid, curr_item: &T| -> Option<T> {
            match origin.get(id) {
                Some(orig_item) if curr_item != orig_item => Some(curr_item.clone()), // Modified
                None => Some(curr_item.clone()),                                      // New
                _ => None,
            }
        };

        match valid_keys {
            // Case A: Filter provided (e.g. based on Graph traversal)
            // We iterate over the VALID keys, not the Map's keys.
            // This implicitly ignores any "excess" objects lingering in the map.
            Some(keys) => keys
                .iter()
                .filter_map(|id| {
                    // We must ensure the item actually exists in the current map
                    self.get(id).and_then(|curr| check_item(id, curr))
                })
                .collect(),

            // Case B: No filter (diff everything in the map)
            None => self
                .iter()
                .filter_map(|(id, curr)| check_item(id, curr))
                .collect(),
        }
    }
}
