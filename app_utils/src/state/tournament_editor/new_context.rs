//! Context for managing the tournament editor state.
//!
//! This module provides a context wrapper around `TournamentEditor` to ensure
//! efficient state updates via `RwSignal` without unnecessary cloning.

use app_core::{
    Group, Stage, TournamentBase, TournamentEditor, TournamentMode,
    utils::validation::ValidationResult,
};
use leptos::prelude::*;
use uuid::Uuid;

/// Context wrapper for `TournamentEditor`.
///
/// Uses an internal `RwSignal` to hold the state, encouraging the use of
/// `.update()` for mutations and `.with()` for reading to minimize cloning of heavy structures.
#[derive(Clone, Copy, Debug)]
pub struct TournamentEditorContext {
    inner: RwSignal<TournamentEditor>,
    /// Indicates if a save or load operation is currently in progress.
    /// Used to disable UI elements across all child components.
    busy: RwSignal<bool>,
    /// Simple counter to trigger URL validation checks manually
    url_validation_trigger: RwSignal<usize>,

    // --- Slices for Tournament Base ---
    /// Read slice for accessing the tournament base name, if any
    pub base_name: Signal<Option<String>>,
    /// Write slice for setting the tournament base name
    pub set_base_name: SignalSetter<String>,
    /// Read slice for accessing the tournament base number of entrants, if any
    pub base_num_entrants: Signal<Option<u32>>,
    /// Write slice for setting the tournament base number of entrants
    pub set_base_num_entrants: SignalSetter<u32>,
    /// Read slice for accessing the tournament base mode, if any
    pub base_mode: Signal<Option<TournamentMode>>,
    /// Write slice for setting the tournament base mode
    pub set_base_mode: SignalSetter<TournamentMode>,
    /// Read slice for accessing the tournament base number of rounds for Swiss System, if any
    pub base_num_rounds_swiss_system: Signal<Option<u32>>,
    /// Write slice for setting the tournament base number of rounds for Swiss System
    pub set_base_num_rounds_swiss_system: SignalSetter<u32>,

    // --- Slices for Current Stage ---
    /// Read slice for accessing the current stage number of groups, if any
    pub stage_num_groups: Signal<Option<u32>>,
    /// Write slice for setting the current stage number of groups
    pub set_stage_num_groups: SignalSetter<u32>,
}

impl Default for TournamentEditorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TournamentEditorContext {
    /// Creates a new, empty context.
    pub fn new() -> Self {
        // core signals
        let inner = RwSignal::new(TournamentEditor::new());
        let busy = RwSignal::new(false);
        let url_validation_trigger = RwSignal::new(0);

        // Create slices for base
        let (base_name, set_base_name) = create_slice(
            inner,
            |inner| inner.get_base().map(|b| b.get_name().to_string()),
            |inner, name: String| {
                inner.get_local_mut().set_base_name(name);
            },
        );
        let (base_num_entrants, set_base_num_entrants) = create_slice(
            inner,
            |inner| inner.get_base().map(|b| b.get_num_entrants()),
            |inner, num_entrants: u32| {
                inner.get_local_mut().set_base_num_entrants(num_entrants);
            },
        );
        let (base_mode, set_base_mode) = create_slice(
            inner,
            |inner| inner.get_base().map(|b| b.get_tournament_mode()),
            |inner, mode: TournamentMode| {
                inner.get_local_mut().set_base_mode(mode);
            },
        );
        let (base_num_rounds_swiss_system, set_base_num_rounds_swiss_system) = create_slice(
            inner,
            |inner| {
                inner
                    .get_base()
                    .and_then(|b| b.get_num_rounds_swiss_system())
            },
            |inner, num_rounds_swiss: u32| {
                inner
                    .get_local_mut()
                    .set_base_num_rounds_swiss_system(num_rounds_swiss);
            },
        );

        // Create slices for stage
        let (stage_num_groups, set_stage_num_groups) = create_slice(
            inner,
            |inner| inner.get_active_stage().map(|s| s.get_num_groups()),
            |inner, num_groups: u32| {
                if let Some(stage) = inner.get_active_stage() {
                    let stage_id = stage.get_id();
                    inner
                        .get_local_mut()
                        .set_stage_number_of_groups(stage_id, num_groups);
                }
            },
        );

        Self {
            // core signals
            inner,
            busy,
            url_validation_trigger,
            // base slices
            base_name,
            set_base_name,
            base_num_entrants,
            set_base_num_entrants,
            base_mode,
            set_base_mode,
            base_num_rounds_swiss_system,
            set_base_num_rounds_swiss_system,
            // stage slices
            stage_num_groups,
            set_stage_num_groups,
        }
    }

    // --- Busy State Management ---

    /// Sets the global busy state of the editor (e.g. during saving).
    pub fn set_busy(&self, is_busy: bool) {
        self.busy.set(is_busy);
    }

    /// Checks if the editor is currently busy (saving/loading).
    pub fn is_busy(&self) -> bool {
        self.busy.get()
    }

    // --- URL Validation Trigger and Navigation ---

    /// Triggers a global check of the current navigation path validity.
    /// This should be called by components after modifying structural data (e.g. changing mode or group counts).
    pub fn trigger_url_validation(&self) {
        self.url_validation_trigger.update(|v| *v += 1);
    }

    /// Returns the trigger signal for effects to listen to.
    /// Use signal with 'track()' to re-run validation when triggered.
    pub fn url_validation_trigger(&self) -> ReadSignal<usize> {
        self.url_validation_trigger.read_only()
    }

    /// Validates the current URL parameters against the editor state.
    pub fn validate_url(
        &self,
        stage_number: Option<u32>,
        group_number: Option<u32>,
        round_number: Option<u32>,
        match_number: Option<u32>,
    ) -> Option<String> {
        self.inner
            .with_untracked(|state| {
                state.validate_object_numbers(
                    stage_number,
                    group_number,
                    round_number,
                    match_number,
                )
            })
            .map(|valid_numbers| {
                valid_numbers
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join("/")
            })
    }

    // --- Actions (Write / Update) ---

    /// Clear the entire editor state.
    pub fn clear(&self) {
        self.inner.update(|state| {
            *state = TournamentEditor::new();
        })
    }

    /// Creates a new tournament base with the given sport ID.
    pub fn new_base(&self, sport_id: Uuid) {
        self.inner.update(|state| {
            state.new_base(sport_id);
        })
    }

    /// Sets tournament configuration based on user input.
    pub fn set_base(&self, base: TournamentBase) {
        self.inner.update(|state| {
            state.set_base(base);
        })
    }

    /// Creates a new stage with the given stage number.
    pub fn new_stage(&self, stage_number: u32) {
        self.inner.update(|state| {
            state.new_stage(stage_number);
        })
    }

    /// Sets stage based on user input.
    pub fn set_stage(&self, stage: Stage) {
        self.inner.update(|state| {
            state.set_stage(stage);
        })
    }

    // --- Selectors (Read / With) ---

    /// Checks if there are unsaved changes.
    pub fn is_changed(&self) -> bool {
        self.inner.with(|state| state.is_changed())
    }

    /// Checks if the current state in the editor is valid according to business rules.
    /// Returns all validation errors, if any.
    /// This should typically be checked before enabling global "Save" or "Publish" actions.
    pub fn validate(&self) -> ValidationResult<()> {
        self.inner.with(|state| state.validation())
    }

    // ToDo: if we create a database port for saving, we do not need these getters anymore
    /// Retrieves the diff of the tournament base for saving.
    pub fn get_base_diff(&self) -> Option<TournamentBase> {
        self.inner.with(|state| state.collect_base_diff())
    }

    /// Retrieves the diff of the stages for saving.
    pub fn get_stages_diff(&self) -> Vec<Stage> {
        self.inner.with(|state| state.collect_stages_diff())
    }

    /// Retrieves the diff of the groups for saving.
    pub fn get_groups_diff(&self) -> Vec<Group> {
        self.inner.with(|state| state.collect_groups_diff())
    }
}
