//! editor module for tournament

use super::*;
use serde::{Deserialize, Serialize};

/// TournamentEditor holds the local editable tournament and the origin tournament for change tracking.
/// This allows tracking changes made to the tournament during editing.
/// The `local` tournament is the one being edited, while the `origin` tournament serves as the reference point
/// to determine what changes have been made.
/// Updating origin will always reset local to origin.
/// Updating local base will not affect origin as long as the tournament ID remains the same.
/// Updating local base of tournament with another ID than origin will reset origin internally to None,
/// indicating that a new tournament is being created.
#[derive(Clone, Deserialize, Serialize)]
pub struct TournamentEditor {
    pub local: Tournament,
    pub origin: Tournament,
}

impl TournamentEditor {
    pub fn new() -> Self {
        Self {
            local: Tournament::new(),
            origin: Tournament::new(),
        }
    }

    // --- Generators for new Tournament Objects ---

    /// Creates a new tournament base in the local state and clears the origin.
    /// Returns the old origin base if any.
    pub fn new_base(&mut self) -> Option<TournamentBase> {
        self.local.new_base();
        self.origin.clear_base()
    }

    pub fn new_stage(&mut self, stage_number: u32) -> bool {
        if let Some(origin_stage) = self.origin.get_stage_by_number(stage_number) {
            // if origin has the stage, we clone it to local

            self.local.set_stage(origin_stage.clone());
            return false;
        }
        self.local.new_stage(stage_number)
    }

    // --- Setters for Tournament Objects after loading from database ---

    /// Sets the tournament base for both origin and local.
    /// Returns the old origin base if any.
    pub fn set_base(&mut self, base: TournamentBase) -> Option<TournamentBase> {
        self.local.set_base(base.clone())?;
        self.origin.set_base(base)
    }

    pub fn set_stage(&mut self, stage: Stage) -> bool {
        if self.origin.set_stage(stage.clone()) {
            return true;
        }
        // since origin is master and successfully set, we force setting local
        // in case of conflicting stage number and id
        if let Some(local_stage) = self.local.get_stage_by_number(stage.get_number())
            && local_stage.get_id() != stage.get_id()
        {
            // remove conflicting stage from local
            self.local.clear_stage(local_stage.get_id());
        }
        self.local.set_stage(stage)
    }

    // --- Getters for reading Tournament Parts ---
    // ToDo: do we need getters for origin?
    pub fn get_origin_base(&self) -> Option<&TournamentBase> {
        self.origin.get_base()
    }

    pub fn get_local_base(&self) -> Option<&TournamentBase> {
        self.local.get_base()
    }

    pub fn get_origin_stage_by_number(&self, stage_number: u32) -> Option<&Stage> {
        self.origin.get_stage_by_number(stage_number)
    }

    pub fn get_local_stage_by_number(&self, stage_number: u32) -> Option<&Stage> {
        self.local.get_stage_by_number(stage_number)
    }

    // --- mut access of local for editing ---
    /// Provides mutable access to the local tournament for direct editing.
    /// Only use this for editing fields or adding new dependent objects.
    /// For setting objects loaded from the database, use the provided setters.
    pub fn get_local_mut(&mut self) -> &mut Tournament {
        &mut self.local
    }

    // --- Diff Collectors for Saving ---
    pub fn collect_base_diff(&self) -> Option<&TournamentBase> {
        self.local
            .get_base()
            .get_diff(&self.origin.get_base(), None)
    }

    pub fn collect_stages_diff(&self) -> Vec<Stage> {
        self.local.stages.get_diff(&self.origin.stages, None)
    }

    pub fn collect_groups_diff(&self) -> Vec<Group> {
        self.local.groups.get_diff(&self.origin.groups, None)
    }

    // --- Change Detection ---

    pub fn is_changed(&self) -> bool {
        self.local.is_changed(&self.origin)
    }

    // --- Validation ---

    pub fn validation(&self) -> ValidationResult<()> {
        // we only validate the local tournament, since origin stems
        // from the DB and is assumed valid
        self.local.validation()
    }

    pub fn validate_object_numbers(
        &self,
        stage_number: Option<u32>,
        group_number: Option<u32>,
        _round_number: Option<u32>,
        _match_number: Option<u32>,
    ) -> Option<Vec<u32>> {
        // we only validate against the local tournament, since it is being edited
        // and changes may invalidate object numbers
        self.local
            .validate_object_numbers(stage_number, group_number, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::id_version::IdVersion;
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
        let mut state = TournamentEditor::new();
        let t = create_test_tournament("My Tournament");

        // Act: Load as origin (like loading from DB)
        state.set_base(t.clone()).unwrap();

        // Assert
        assert!(
            state.origin.get_base().is_some(),
            "Tournament should be set"
        );
        assert!(state.origin.get_base().is_some(), "Origin should be set");
        assert_eq!(
            state.local.get_base(),
            state.origin.get_base(),
            "Current and Origin should be identical"
        );
        assert!(
            !state.local.is_changed(&state.origin),
            "State should not be marked as changed initially"
        );
        assert!(
            state.local.collect_base_diff(&state.origin).is_none(),
            "Diff should be None"
        );
    }

    #[test]
    fn test_update_tournament_detects_changes() {
        // Arrange
        let mut state = TournamentEditor::new();
        let original = create_test_tournament("Original Name");
        state.set_base(original.clone()).unwrap();

        // Act: Simulate User Edit
        state.get_local_mut().set_base_name("Changed Name");

        // Assert
        assert!(
            state.local.is_changed(&state.origin),
            "State should be marked as changed"
        );

        let diff = state.local.collect_base_diff(&state.origin);
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
        let mut state = TournamentEditor::new();
        let original = create_test_tournament("Base");
        state.set_base(original.clone()).unwrap();

        // Act 1: Modify
        state.get_local_mut().set_base_name("Modified");
        // Assert
        assert!(
            state.local.is_changed(&state.origin),
            "Should be dirty after edit"
        );

        // Act 2: Revert (Set back to original values manually via UI logic)
        state.get_local_mut().set_base_name(original.get_name());
        // Assert
        assert!(
            !state.local.is_changed(&state.origin),
            "Should be clean after reverting values"
        );
        assert!(state.local.collect_base_diff(&state.origin).is_none());
    }

    #[test]
    fn test_save_sync_updates_origin() {
        // Arrange
        let mut state = TournamentEditor::new();
        let t_v1 = create_test_tournament("Version 1");
        state.set_base(t_v1.clone()).unwrap();

        // Modify local state
        state.get_local_mut().set_base_name("Version 2 Draft");

        assert!(
            state.local.is_changed(&state.origin),
            "Pre-condition: State is dirty"
        );

        // Act: Simulate successful save response from server (Version 2)
        // This simulates the Resource effect triggering updates
        let diff = state.collect_base_diff().unwrap().clone();

        // Ideally version number increments here in real DB logic
        state.set_base(diff).unwrap();

        // Assert
        assert!(
            !state.local.is_changed(&state.origin),
            "State should be clean after syncing with server response"
        );
        assert_eq!(
            state.origin.get_base().unwrap().get_name(),
            "Version 2 Draft"
        );
        assert_eq!(
            state.local.get_base().unwrap().get_name(),
            "Version 2 Draft"
        );
    }
}
