//! Context for managing the tournament editor state.
//!
//! This module provides a context wrapper around `TournamentEditor` to ensure
//! efficient state updates via `RwSignal` without unnecessary cloning.

pub mod base;
pub mod stage;

use crate::{
    hooks::use_url_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::{GroupNumberParams, ParamQuery, StageNumberParams, TournamentBaseIdQuery},
    state::{
        EditorContext, EditorContextWithResource, SimpleEditorOptions, toast_state::ToastContext,
    },
};
use app_core::{Stage, Tournament, TournamentBase};
use base::{BaseEditorContext, BaseEditorContextOptions};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use stage::{StageEditorContext, StageEditorContextOptions};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct TournamentEditorContext {
    // --- state & derived signals ---
    /// The local editable tournament
    local: RwSignal<Option<Tournament>>,
    /// keep owner for creating new object context signals in the editor context
    owner: StoredValue<Owner>,
    /// Base editor context for the tournament, managing the base tournament object
    pub base_editor: BaseEditorContext,
    /// Map of stage editors for the stages of the tournament, keyed by stage number
    stage_editors: RwSignal<HashMap<u32, StageEditorContext>>,
}

impl EditorContext for TournamentEditorContext {
    // we use TournamentBase as object type, since Tournament itself is a meta object, which only exists
    // in the editor state and is not stored in the database. Since TournamentBase is the root object
    // of a tournament, we use it to identify the tournament in the editor context and for loading
    // and listing tournaments from the server.
    type ObjectType = TournamentBase;
    type NewEditorOptions = SimpleEditorOptions;

    fn new(options: SimpleEditorOptions) -> Self {
        // --- navigation and globale state context ---
        let navigate = use_navigate();
        let UseQueryNavigationReturn {
            url_update_path, ..
        } = use_query_navigation();
        let toast_ctx = expect_context::<ToastContext>();

        // --- core signals ---
        let local = RwSignal::new(None::<Tournament>);

        let owner = StoredValue::new(
            Owner::current().expect("TournamentEditorContext must be created within a component"),
        );

        let base_editor_options = BaseEditorContextOptions {
            object_id: options.object_id,
            local_tournament: local,
        };
        let base_editor = BaseEditorContext::new(base_editor_options);
        let stage_editors = RwSignal::new(HashMap::new());

        // --- url parameters & queries & validation ---
        let tournament_base_id = TournamentBaseIdQuery::use_param_query();
        let active_stage_number = StageNumberParams::use_param_query();
        let active_group_number = GroupNumberParams::use_param_query();

        let valid_object_numbers = Memo::new(move |_| {
            local.with(|may_be_t| {
                may_be_t.as_ref().and_then(|t| {
                    t.validate_object_numbers(
                        active_stage_number.get(),
                        active_group_number.get(),
                        None,
                        None,
                    )
                })
            })
        });
        // Effect to update URL if invalid object numbers are detected
        Effect::new({
            let navigate = navigate.clone();
            move || {
                // Validate url against current params and navigate if invalid params detected
                if let Some(selected_base_id) = tournament_base_id.get()
                    && let Some(tournament_id) = base_editor.id.get()
                    && tournament_id == selected_base_id
                    && let Some(von) = valid_object_numbers.get()
                {
                    let toast_msg = match von.len() {
                        0 => format!(
                            "Invalid stage number '{}' in URL, navigating back to base view",
                            active_stage_number.get().unwrap_or_default()
                        ),
                        1 => format!(
                            "Invalid group number '{}' in URL, navigating back to stage view",
                            active_group_number.get().unwrap_or_default()
                        ),
                        _ => "Navigated to corrected URL with valid object numbers".to_string(),
                    };
                    toast_ctx.error(toast_msg, None);
                    // Build redirect path from valid object numbers
                    let redirect_path = von
                        .iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join("/");
                    let redirect_path = if redirect_path.is_empty() {
                        "/tournaments/edit".to_string()
                    } else {
                        format!("/tournaments/edit/{redirect_path}")
                    };
                    // Navigate to the corrected path
                    let url = url_update_path(&redirect_path);
                    navigate(
                        &url,
                        NavigateOptions {
                            replace: true, // Replace history to avoid dead ends
                            scroll: false,
                            ..Default::default()
                        },
                    );
                }
            }
        });

        Self {
            local,
            owner,
            base_editor,
            stage_editors,
        }
    }

    /// Set the current tournament base in the editor context, updating all relevant state accordingly.
    fn set_object(&self, tournament_base: TournamentBase) {
        self.base_editor.set_object(tournament_base);
    }

    /// Create a new tournament base object in the editor context, returning its unique identifier.
    fn new_object(&self) -> Option<Uuid> {
        self.base_editor.new_object();
        self.base_editor.id.get()
    }
}

impl TournamentEditorContext {
    pub fn update_base_in_editor(&self, base: &TournamentBase) {
        let optimistic_version = self.base_editor.optimistic_version_signal().get_untracked();
        if optimistic_version.is_none() {
            self.base_editor.set_object(base.clone());
        }
        if let Some(ov) = optimistic_version
            && ov < base.get_version().unwrap_or_default()
        {
            self.base_editor.set_object(base.clone());
        }
    }

    pub fn spawn_stage_editor(
        &self,
        object_id: Option<Uuid>,
        stage_number: u32,
    ) -> Option<StageEditorContext> {
        if let Some(stage_editor) = self.get_stage_editor(stage_number) {
            return Some(stage_editor); // Editor already exists for this stage number
        }
        if self.local.with(|may_be_t| may_be_t.is_none()) {
            return None; // No Tournament loaded in local editor state, cannot prepare stage
        }
        let stage_editor_options = StageEditorContextOptions {
            stage_number,
            object_id,
            local_tournament: self.local,
        };
        let stage_editor = self
            .owner
            .with_value(|owner| owner.with(|| StageEditorContext::new(stage_editor_options)));
        self.stage_editors.update(|editors| {
            editors.insert(stage_number, stage_editor);
        });
        Some(stage_editor)
    }

    pub fn get_stage_editor(&self, stage_number: u32) -> Option<StageEditorContext> {
        self.stage_editors
            .with(|editors| editors.get(&stage_number).copied())
    }

    pub fn update_stage_in_editor(&self, stage: &Stage) {
        let Some(stage_editor) = self.get_stage_editor(stage.get_number()) else {
            return; // No editor for this stage number, cannot update
        };
        let optimistic_version = stage_editor.optimistic_version_signal().get_untracked();
        if optimistic_version.is_none() {
            stage_editor.set_object(stage.clone());
        }
        if let Some(ov) = optimistic_version
            && ov < stage.get_version().unwrap_or_default()
        {
            stage_editor.set_object(stage.clone());
        }
    }

    /// creates a new stage editor for the given stage number if it does not exist yet
    /// and initializes it with a new stage object.
    pub fn prepare_stage(&self, stage_number: u32) {
        if self.get_stage_editor(stage_number).is_some() {
            return; // Editor already exists for this stage number, nothing to prepare
        }
        if let Some(stage_editor) = self.spawn_stage_editor(None, stage_number) {
            // create new stage object in editor
            stage_editor.new_object();
        }
    }

    pub fn prepare_group(&self, _stage_number: u32, _group_number: u32) {
        // ToDo: implement group editor context and insert into map here
    }
}
