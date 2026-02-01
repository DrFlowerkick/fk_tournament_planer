//! editor module for tournament

use super::*;
use serde::{Deserialize, Serialize};

/// State of the tournament editor, indicating whether a new tournament is being created
/// or an existing tournament is being edited.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TournamentEditorState {
    /// Initial state with no tournament loaded.
    None,
    /// New tournament being created (no origin).
    New,
    /// Existing tournament being edited (has origin).
    Edit,
}

/// TournamentEditor holds the local editable tournament and the origin tournament for change tracking.
/// This allows tracking changes made to the tournament during editing.
/// The `local` tournament is the one being edited, while the `origin` tournament serves as the reference point
/// to determine what changes have been made.
/// Updating origin will always reset local to origin.
/// Updating local base will not affect origin as long as the tournament ID remains the same.
/// Updating local base of tournament with another ID than origin will reset origin internally to None,
/// indicating that a new tournament is being created.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TournamentEditor {
    local: Tournament,
    origin: Tournament,
    active_stage_id: Option<Uuid>,
    active_group_id: Option<Uuid>,
    active_round_id: Option<Uuid>,
    active_match_id: Option<Uuid>,
}

impl TournamentEditor {
    pub fn new() -> Self {
        Self {
            local: Tournament::new(),
            origin: Tournament::new(),
            active_stage_id: None,
            active_group_id: None,
            active_round_id: None,
            active_match_id: None,
        }
    }

    // --- State of Editor ---
    /// Returns the current state of the editor: None, New, or Edit.
    pub fn get_state(&self) -> TournamentEditorState {
        if self.origin.get_base().is_none() {
            if self.local.get_base().is_none() {
                TournamentEditorState::None
            } else {
                TournamentEditorState::New
            }
        } else {
            TournamentEditorState::Edit
        }
    }

    // --- Generators for new Tournament Objects ---

    /// Creates a new tournament base in the local state and clears the origin.
    /// Returns the old origin base if any.
    pub fn new_base(&mut self, sport_id: Uuid) -> Option<TournamentBase> {
        // store old origin
        let old_origin = self.origin.clear_base();
        // reset self
        *self = TournamentEditor::new();
        // create new base in local
        self.local.new_base(sport_id);
        old_origin
    }

    pub fn new_stage(&mut self, stage_number: u32) -> bool {
        let Some(base_id) = self.local.get_base().map(|b| b.get_id()) else {
            // cannot create stage without base
            return false;
        };
        if let Some(origin_stage) = self.origin.get_stage_by_number(stage_number)
            && origin_stage.get_tournament_id() == base_id
        {
            // if origin has the stage, we clone it to local
            self.local.set_stage(origin_stage.clone());
            self.active_stage_id = Some(origin_stage.get_id());
            return false;
        }
        // else we create a new stage in local
        if self.local.new_stage(stage_number) {
            return true;
        }
        // set active stage id
        self.active_stage_id = self
            .local
            .get_stage_by_number(stage_number)
            .map(|s| s.get_id());
        false
    }

    // --- Setters for Tournament Objects after loading from database ---

    /// Sets the tournament base for both origin and local.
    /// Returns the old origin base if any.
    pub fn set_base(&mut self, base: TournamentBase) -> Option<TournamentBase> {
        self.local.set_base(base.clone());
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
        self.active_stage_id = Some(stage.get_id());
        self.local.set_stage(stage)
    }

    // --- Getters for reading Tournament Parts ---
    pub fn get(&self) -> &Tournament {
        &self.local
    }

    pub fn get_base(&self) -> Option<&TournamentBase> {
        self.local.get_base()
    }

    pub fn get_active_stage_id(&self) -> Option<Uuid> {
        self.active_stage_id
    }

    pub fn get_active_stage(&self) -> Option<&Stage> {
        self.active_stage_id
            .and_then(|id| self.local.get_stage_by_id(id))
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
        self.local.collect_base_diff(&self.origin)
    }

    pub fn collect_stages_diff(&self) -> Vec<Stage> {
        self.local.collect_stages_diff(&self.origin)
    }

    pub fn collect_groups_diff(&self) -> Vec<Group> {
        self.local.collect_groups_diff(&self.origin)
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
        state.set_base(t.clone());

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
        state.set_base(original.clone());

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
        state.set_base(original.clone());

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
        state.set_base(t_v1.clone());

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
        state.set_base(diff);

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

    #[test]
    fn test_serde_tournament_editor() {
        let mut tournament_editor = TournamentEditor::new();
        let sport_id = Uuid::new_v4();
        tournament_editor.new_base(sport_id);
        tournament_editor
            .get_local_mut()
            .set_base_name("Test Tournament");
        tournament_editor.get_local_mut().set_base_num_entrants(16);
        tournament_editor
            .get_local_mut()
            .set_base_mode(TournamentMode::PoolAndFinalStage);

        tournament_editor.new_stage(0);
        let stage_0_id = tournament_editor.get_active_stage_id();
        tournament_editor.new_stage(1);
        let stage_1_id = tournament_editor.get_active_stage_id();

        tournament_editor
            .get_local_mut()
            .set_stage_number_of_groups(stage_0_id.unwrap(), 4);
        tournament_editor
            .get_local_mut()
            .set_stage_number_of_groups(stage_1_id.unwrap(), 2);

        println!(
            "{}",
            serde_json::to_string_pretty(&tournament_editor).unwrap()
        );
        let serialized = serde_json::to_string(&tournament_editor).unwrap();
        let deserialized: TournamentEditor = serde_json::from_str(&serialized).unwrap();

        assert_eq!(tournament_editor.get_base(), deserialized.get_base());
        assert_eq!(
            tournament_editor.local.stages.len(),
            deserialized.local.stages.len()
        );
        for (id, stage) in &tournament_editor.local.stages {
            let deserialized_stage = deserialized.local.stages.get(id).unwrap();
            assert_eq!(stage, deserialized_stage);
        }
    }

    #[test]
    fn test_debug_deserialization() {
        let data = r#"{"local":{"base":{"id_version":{"Existing":{"id":"6ad2ec97-8899-4a4a-9ef1-36bd3097d174","version":1}},"name":"Test Name 03","sport_id":"f802dcb7-24e0-5f79-867c-ef4c477311f6","num_entrants":10,"t_type":"Scheduled","mode":"PoolAndFinalStage","state":"Draft"},"structure":{"nodes":["6ad2ec97-8899-4a4a-9ef1-36bd3097d174","ce3799c3-d978-444e-a9ae-5c91117d4350","fbf1aa08-cfd2-4409-85de-37b27bd5244d"],"node_holes":[],"edge_property":"directed","edges":[[0,1,"Stage"],[0,2,"Stage"]]},"stages":{"ce3799c3-d978-444e-a9ae-5c91117d4350":{"id_version":{"NewWithId":"ce3799c3-d978-444e-a9ae-5c91117d4350"},"tournament_id":"6ad2ec97-8899-4a4a-9ef1-36bd3097d174","number":0,"num_groups":4},"fbf1aa08-cfd2-4409-85de-37b27bd5244d":{"id_version":{"NewWithId":"fbf1aa08-cfd2-4409-85de-37b27bd5244d"},"tournament_id":"6ad2ec97-8899-4a4a-9ef1-36bd3097d174","number":1,"num_groups":2}},"groups":{}},"origin":{"base":{"id_version":{"Existing":{"id":"6ad2ec97-8899-4a4a-9ef1-36bd3097d174","version":1}},"name":"Test Name 03","sport_id":"f802dcb7-24e0-5f79-867c-ef4c477311f6","num_entrants":10,"t_type":"Scheduled","mode":"SingleStage","state":"Draft"},"structure":{"nodes":["6ad2ec97-8899-4a4a-9ef1-36bd3097d174"],"node_holes":[],"edge_property":"directed","edges":[]},"stages":{},"groups":{}},"active_stage_id":"fbf1aa08-cfd2-4409-85de-37b27bd5244d","active_group_id":null,"active_round_id":null,"active_match_id":null}"#;
        let res: Result<TournamentEditor, _> = serde_json::from_str(data);
        assert!(res.is_ok(), "Fehler: {:?}", res.err());
    }
}
