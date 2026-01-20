//! manage global state for tournament editor

use app_core::{
    Group, Stage, TournamentBase,
    utils::traits::{ObjectIdVersion, ObjectNumber},
};
use leptos::logging::warn;
use petgraph::{Direction, graphmap::DiGraphMap, visit::Bfs};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

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

// --- State Definition ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    Stage,
    Group,
}

#[derive(Clone)]
pub struct TournamentEditorState {
    /// new / edited tournament
    pub tournament: Option<TournamentBase>,
    /// origin tournament from server (snapshot for dirty check)
    pub origin_tournament: Option<TournamentBase>,
    /// map of tournament dependencies
    pub structure: DiGraphMap<Uuid, DependencyType>,
    /// stages associated with the tournament
    pub stages: HashMap<Uuid, Stage>,
    /// origin stages from server
    pub origin_stages: HashMap<Uuid, Stage>,
    /// groups associated with stages (not yet used)
    pub groups: HashMap<Uuid, Group>,
    /// origin groups from server (not yet used)
    pub origin_groups: HashMap<Uuid, Group>,
}

impl TournamentEditorState {
    pub fn new() -> Self {
        TournamentEditorState {
            tournament: None,
            origin_tournament: None,
            structure: DiGraphMap::new(),
            stages: HashMap::new(),
            origin_stages: HashMap::new(),
            groups: HashMap::new(),
            origin_groups: HashMap::new(),
        }
    }

    // ---- Setters ----

    /// Unified setter for the tournament base.
    /// Handles context switching (loading a different tournament) and performs
    /// cleanup of dependent objects if business rules require it (only when editing).
    pub fn set_tournament(&mut self, tournament: TournamentBase, is_origin: bool) {
        let Some(new_id) = tournament.get_id_version().get_id() else {
            warn!("TournamentBase has no ID, cannot set tournament state");
            return;
        };

        // Ensure Graph Node exists
        self.structure.add_node(new_id);

        // Assign new tournament state
        if is_origin {
            // Case: Loading from DB or Post-Save update.
            // We assume DB state is valid.
            self.origin_tournament = Some(tournament.clone());
        }
        self.tournament = Some(tournament);

        // Validation: Check if changes invalidate child objects (e.g. Mode change -> fewer stages)
        self.cleanup_excess_stages(new_id);
    }

    /// Adds a stage to the state and links it to the tournament.
    pub fn set_stage(&mut self, stage: Stage, is_origin: bool) {
        let Some(stage_id) = stage.get_id_version().get_id() else {
            warn!("Stage has no ID, cannot add to tournament editor state");
            return;
        };
        let Some(tournament_id) = self.get_root_id() else {
            warn!("TournamentBase has no ID, cannot add stage to state");
            return;
        };

        // Link to tournament root, which although adds the node if missing
        self.structure
            .add_edge(tournament_id, stage_id, DependencyType::Stage);

        // Add to stages map
        if is_origin {
            // Case: Loading from DB or Post-Save update.
            // We assume DB state is valid.
            self.origin_stages.insert(stage_id, stage.clone());
        }
        self.stages.insert(stage_id, stage);

        // Validation: Check if changes invalidate child objects (e.g. fewer groups)
        self.cleanup_excess_groups(stage_id);
    }

    // --- Getters for keeping state of new tournament & dependencies ---
    pub fn get_tournament(&self) -> Option<&TournamentBase> {
        self.tournament.as_ref()
    }

    pub fn get_stage_by_number(&self, stage_number: u32) -> Option<&Stage> {
        let Some(start) = self.get_root_id() else {
            return None;
        };
        for (_source, target, edge) in self.structure.edges_directed(start, Direction::Outgoing) {
            if let DependencyType::Stage = *edge
                && let Some(stage) = self.stages.get(&target)
                && stage.get_number() == stage_number
            {
                return Some(stage);
            }
        }
        None
    }

    // --- Getters for Saving ---

    pub fn get_tournament_diff(
        &self,
    ) -> <Option<TournamentBase> as Diffable<TournamentBase>>::Diff {
        // Option diffing ignores the filter
        self.tournament.get_diff(&self.origin_tournament, None)
    }

    /// Returns modified or new stages that are currently linked in the graph structure.
    pub fn get_stages_diff(&self) -> <HashMap<Uuid, Stage> as Diffable<Stage>>::Diff {
        // We collect ALL valid reachable IDs. The Diffable impl for HashMap will pick
        // only the ones that exist in the 'stages' map.
        let valid_ids = self.get_valid_dependencies();

        self.stages.get_diff(&self.origin_stages, Some(&valid_ids))
    }

    pub fn get_groups_diff(&self) -> <HashMap<Uuid, Group> as Diffable<Group>>::Diff {
        // We collect ALL valid reachable IDs. The Diffable impl for HashMap will pick
        // only the ones that exist in the 'groups' map.
        let valid_ids = self.get_valid_dependencies();

        self.groups.get_diff(&self.origin_groups, Some(&valid_ids))
    }

    // --- Change Detection ---
    
    /// Checks if there are any changes compared to the origin state.
    pub fn is_changed(&self) -> bool {
        let Some(start) = self.get_root_id() else {
            return false;
        };

        // Check root
        if self.tournament != self.origin_tournament {
            return true;
        }

        // Traverse structure
        let mut bfs = Bfs::new(&self.structure, start);
        while let Some(object) = bfs.next(&self.structure) {
            for (_source, target, edge) in
                self.structure.edges_directed(object, Direction::Outgoing)
            {
                match edge {
                    DependencyType::Stage => {
                        let curr = self.stages.get(&target);
                        let orig = self.origin_stages.get(&target);
                        if curr != orig {
                            return true;
                        }
                    }
                    DependencyType::Group => {
                        let curr = self.groups.get(&target);
                        let orig = self.origin_groups.get(&target);
                        if curr != orig {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // --- Validation ---

    /// Validates the entire currently loaded tournament structure.
    /// Returns `true` if the entire structure represents a valid state that could be saved/started.
    pub fn is_valid(&self) -> bool {
        // 1. Root Tournament Check
        let Some(tournament) = &self.tournament else {
            return false;
        };

        // Assuming TournamentBase has a validate() method returning Result
        if tournament.validate().is_err() {
            return false;
        }

        let Some(start) = self.get_root_id() else {
            return false;
        };

        // Traverse structure
        let mut bfs = Bfs::new(&self.structure, start);
        while let Some(object) = bfs.next(&self.structure) {
            for (_source, target, edge) in
                self.structure.edges_directed(object, Direction::Outgoing)
            {
                match edge {
                    DependencyType::Stage => {
                        // Stage needs Tournament context for validation (e.g. strict entrant limits)
                        if let Some(stage) = self.stages.get(&target)
                            && stage.validate(tournament).is_err()
                        {
                            return false;
                        };
                    }
                    DependencyType::Group => {
                        // ToDo: implement group validation
                        if let Some(_group) = self.groups.get(&target) {
                            continue;
                        }
                    }
                }
            }
        }

        true
    }

    // --- Helpers ---

    /// Returns the root tournament ID, preferring the origin tournament if available
    fn get_root_id(&self) -> Option<Uuid> {
        // prefer origin ID as anchor, fallback to current
        self.origin_tournament
            .as_ref()
            .and_then(|t| t.get_id_version().get_id())
            .or_else(|| {
                self.tournament
                    .as_ref()
                    .and_then(|t| t.get_id_version().get_id())
            })
    }

    /// Checks if the new tournament configuration requires removing stages.
    fn cleanup_excess_stages(&mut self, root_id: Uuid) {
        if let Some(tournament) = &self.tournament {
            let num_expected = tournament.get_tournament_mode().get_num_of_stages();

            let excess_ids =
                self.collect_excess_ids(root_id, DependencyType::Stage, &self.stages, num_expected);

            for stage_id in excess_ids {
                // We only remove the graph edge. The object remains in the Map until strictly cleared,
                // or we could remove it here. Removing edge hides it from the UI traversal.
                self.structure.remove_edge(root_id, stage_id);
            }
        }
    }

    /// Checks if the new tournament configuration requires removing stages.
    fn cleanup_excess_groups(&mut self, root_id: Uuid) {
        if let Some(stage) = self.stages.get(&root_id) {
            let num_expected = stage.get_num_groups();

            let excess_ids =
                self.collect_excess_ids(root_id, DependencyType::Group, &self.groups, num_expected);

            for group_id in excess_ids {
                // We only remove the graph edge. The object remains in the Map until strictly cleared,
                // or we could remove it here. Removing edge hides it from the UI traversal.
                self.structure.remove_edge(root_id, group_id);
            }
        }
    }

    /// Collects IDs of dependent objects of a specific type that exceed a given limit.
    fn collect_excess_ids<O>(
        &self,
        parent_id: Uuid,
        dep_type: DependencyType,
        store: &HashMap<Uuid, O>,
        limit: u32,
    ) -> Vec<Uuid>
    where
        O: ObjectNumber + ObjectIdVersion,
    {
        self.structure
            .edges_directed(parent_id, Direction::Outgoing)
            .filter(|(_, _, e)| **e == dep_type)
            .filter_map(|(_, target, _)| store.get(&target).map(|obj| (target, obj)))
            .filter(|(_, obj)| obj.get_object_number() >= limit)
            .map(|(id, _)| id)
            .collect()
    }

    /// Collects all valid IDs of a specific type reachable from the root tournament
    fn get_valid_dependencies(&self) -> HashSet<Uuid> {
        let mut valid = HashSet::new();
        let Some(start) = self.get_root_id() else {
            return valid;
        };

        // Use BFS to traverse the entire dependency graph starting from root
        let mut bfs = Bfs::new(&self.structure, start);
        while let Some(node) = bfs.next(&self.structure) {
            // We include all reachable nodes.
            // Since UUIDs are unique, we don't need to filter by type here.
            // The specific HashMaps (stages, groups, etc.) will simply ignore
            // IDs that belong to other types when we try to look them up later.
            valid.insert(node);
        }

        valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::utils::id_version::IdVersion;
    use app_core::{TournamentBase, TournamentMode, TournamentState, TournamentType};
    use uuid::Uuid;

    /// Helper to create a valid tournament instance for testing using public API.
    fn create_test_tournament(name: &str) -> TournamentBase {
        let id = Uuid::new_v4();
        let mut t = TournamentBase::default();

        // Use public setters to configure the object
        t.set_id_version(IdVersion::NewWithId(id));
        t.set_name(name);
        t.set_sport_id(Uuid::new_v4());
        t.set_num_entrants(10);
        t.set_tournament_type(TournamentType::Scheduled);
        t.set_tournament_mode(TournamentMode::SingleStage);
        t.set_tournament_state(TournamentState::Draft);

        t
    }

    #[test]
    fn test_new_tournament_load_is_clean() {
        // Arrange
        let mut state = TournamentEditorState::new();
        let t = create_test_tournament("My Tournament");

        // Act: Load as origin (like loading from DB)
        state.set_tournament(t.clone(), true);

        // Assert
        assert!(state.tournament.is_some(), "Tournament should be set");
        assert!(state.origin_tournament.is_some(), "Origin should be set");
        assert_eq!(
            state.tournament, state.origin_tournament,
            "Current and Origin should be identical"
        );
        assert!(
            !state.is_changed(),
            "State should not be marked as changed initially"
        );
        assert!(state.get_tournament_diff().is_none(), "Diff should be None");
    }

    #[test]
    fn test_update_tournament_detects_changes() {
        // Arrange
        let mut state = TournamentEditorState::new();
        let original = create_test_tournament("Original Name");
        state.set_tournament(original.clone(), true);

        // Act: Simulate User Edit (is_origin = false)
        let mut modified = original.clone();
        modified.set_name("Changed Name");
        state.set_tournament(modified.clone(), false);

        // Assert
        assert!(state.is_changed(), "State should be marked as changed");

        let diff = state.get_tournament_diff();
        assert!(diff.is_some(), "Diff should return a value");
        assert_eq!(
            diff.unwrap().get_name(),
            "Changed Name",
            "Diff should contain the updated name"
        );
    }

    #[test]
    fn test_revert_changes_clears_dirty_state() {
        // Arrange
        let mut state = TournamentEditorState::new();
        let original = create_test_tournament("Base");
        state.set_tournament(original.clone(), true);

        // Act 1: Modify
        let mut modified = original.clone();
        modified.set_name("Modified");
        state.set_tournament(modified.clone(), false);
        assert!(state.is_changed(), "Should be dirty after edit");

        // Act 2: Revert (Set back to original values manually via UI logic)
        state.set_tournament(original.clone(), false);

        // Assert
        assert!(
            !state.is_changed(),
            "Should be clean after reverting values"
        );
        assert!(state.get_tournament_diff().is_none());
    }

    #[test]
    fn test_save_sync_updates_origin() {
        // Arrange
        let mut state = TournamentEditorState::new();
        let t_v1 = create_test_tournament("Version 1");
        state.set_tournament(t_v1.clone(), true);

        // Modify local state
        let mut t_edit = t_v1.clone();
        t_edit.set_name("Version 2 Draft");
        state.set_tournament(t_edit.clone(), false);

        assert!(state.is_changed(), "Pre-condition: State is dirty");

        // Act: Simulate successful save response from server (Version 2)
        // This simulates the Resource effect triggering updates
        let t_v2 = t_edit.clone();
        // Ideally version number increments here in real DB logic
        state.set_tournament(t_v2.clone(), true);

        // Assert
        assert!(
            !state.is_changed(),
            "State should be clean after syncing with server response"
        );
        assert_eq!(
            state.origin_tournament.unwrap().get_name(),
            "Version 2 Draft"
        );
        assert_eq!(state.tournament.unwrap().get_name(), "Version 2 Draft");
    }
}
