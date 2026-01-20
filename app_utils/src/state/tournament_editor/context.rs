//! Context for managing the tournament editor state.
//!
//! This module provides a context wrapper around `TournamentEditorState` to ensure
//! efficient state updates via `RwSignal` without unnecessary cloning.

use super::state::TournamentEditorState;
use app_core::{Stage, TournamentBase};
use leptos::prelude::*;

/// Context wrapper for `TournamentEditorState`.
///
/// Uses an internal `RwSignal` to hold the state, encouraging the use of
/// `.update()` for mutations and `.with()` for reading to minimize cloning of heavy structures.
#[derive(Clone, Copy, Debug)]
pub struct TournamentEditorContext {
    inner: RwSignal<TournamentEditorState>,
    /// Indicates if a save or load operation is currently in progress.
    /// Used to disable UI elements across all child components.
    busy: RwSignal<bool>,
}

impl Default for TournamentEditorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TournamentEditorContext {
    /// Creates a new, empty context.
    pub fn new() -> Self {
        Self {
            inner: RwSignal::new(TournamentEditorState::new()),
            busy: RwSignal::new(false),
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

    // --- Actions (Write / Update) ---

    /// Sets tournament configuration based on user input.
    pub fn set_tournament(&self, tournament: TournamentBase, is_origin: bool) {
        self.inner.update(|state| {
            state.set_tournament(tournament, is_origin);
        });
    }

    /// Sets stage based on user input.
    pub fn set_stage(&self, stage: Stage, is_origin: bool) {
        self.inner.update(|state| {
            state.set_stage(stage, is_origin);
        });
    }

    // --- Selectors (Read / With) ---

    /// Checks if there are unsaved changes.
    pub fn is_changed(&self) -> bool {
        self.inner.with(|state| state.is_changed())
    }

    /// Returns the current tournament for display.
    pub fn get_tournament(&self) -> Option<TournamentBase> {
        self.inner.with(|state| state.get_tournament().cloned())
    }

    pub fn get_stage_by_number(&self, stage_number: u32) -> Option<Stage> {
        self.inner
            .with(|state| state.get_stage_by_number(stage_number).cloned())
    }

    /// Retrieves the diff of the tournament base for saving.
    pub fn get_tournament_diff(&self) -> Option<TournamentBase> {
        self.inner.with(|state| state.get_tournament_diff())
    }

    /// Retrieves the diff of the stages for saving.
    pub fn get_stages_diff(&self) -> Vec<Stage> {
        self.inner.with(|state| state.get_stages_diff())
    }

    /// Retrieves the diff of the groups for saving.
    pub fn get_groups_diff(&self) -> Vec<app_core::Group> {
        self.inner.with(|state| state.get_groups_diff())
    }
}
