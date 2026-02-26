//! Context for managing the tournament editor state.
//!
//! This module provides a context wrapper around `TournamentEditor` to ensure
//! efficient state updates via `RwSignal` without unnecessary cloning.

pub mod base;
pub mod stage;

use crate::{
    hooks::use_query_navigation::{
        MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
    },
    params::{GroupNumberParams, ParamQuery, StageNumberParams},
    state::{EditorContext, EditorContextWithResource, SimpleEditorOptions},
};
use app_core::{Tournament, TournamentBase};
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
    /// The original tournament loaded from storage.
    origin: RwSignal<Option<Tournament>>,
    /// Readonly signal for the original tournament, to be used for comparison in validation and update checks
    origin_readonly: Signal<Option<Tournament>>,
    // ToDo: wrap into RwSignal?
    pub base_editor: BaseEditorContext,
    // ToDo: change into RwSignal?
    stage_editors: StoredValue<HashMap<u32, StageEditorContext>>,
}

impl EditorContext for TournamentEditorContext {
    type ObjectType = Tournament;
    type NewEditorOptions = SimpleEditorOptions;

    fn new(options: SimpleEditorOptions) -> Self {
        // --- navigation and globale state context ---
        let navigate = use_navigate();
        let UseQueryNavigationReturn {
            url_matched_route, ..
        } = use_query_navigation();

        // --- core signals ---
        let local = RwSignal::new(None::<Tournament>);
        let origin = RwSignal::new(None::<Tournament>);

        let base_editor_options = BaseEditorContextOptions {
            object_id: options.object_id,
            local_tournament: local,
            origin_tournament: origin,
        };
        let base_editor = BaseEditorContext::new(base_editor_options);
        let stage_editors = StoredValue::new(HashMap::new());

        // --- url parameters & queries & validation ---
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
                if let Some(von) = valid_object_numbers.get() {
                    // Build redirect path from valid object numbers
                    let redirect_path = von
                        .iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join("/");
                    // Navigate to the corrected path
                    // ToDo: matched route may not be correct, depending on where on route we are.
                    let url = url_matched_route(MatchedRouteHandler::Extend(&redirect_path));
                    // ToDo: remove debug:
                    leptos::logging::debug_log!(
                        "Redirecting to {} due to invalid URL parameters",
                        url
                    );
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
            origin,
            origin_readonly: origin.into(),
            base_editor,
            stage_editors,
        }
    }

    /// Get the original tournament currently loaded in the editor context, if any.
    fn origin_signal(&self) -> Signal<Option<Tournament>> {
        self.origin_readonly
    }

    /// Set the current tournament in the editor context, updating all relevant state accordingly.
    fn set_object(&self, tournament: Tournament) {
        self.local.set(Some(tournament.clone()));
        self.origin.set(Some(tournament.clone()));
        self.base_editor.set_object(tournament.get_base().clone());
    }

    /// Create a new tournament object in the editor context, returning its unique identifier.
    fn new_object(&self) -> Option<Uuid> {
        self.base_editor.new_object();
        self.base_editor.id.get()
    }
}

impl TournamentEditorContext {
    pub fn update_base_in_editor(&self, base: &TournamentBase) {
        let optimistic_version = self.base_editor.optimistic_version_signal().get();
        if optimistic_version.is_none() {
            self.base_editor.set_object(base.clone());
        }
        if let Some(ov) = optimistic_version
            && ov < base.get_version().unwrap_or_default()
        {
            self.base_editor.set_object(base.clone());
        }
    }

    pub fn prepare_stage(&self, stage_number: u32) {
        if self
            .stage_editors
            .with_value(|editors| editors.contains_key(&stage_number))
        {
            return; // Editor already exists for this stage number
        }
        self.local.update(|te| {
            if let Some(t) = te.as_mut() {
                t.new_stage(stage_number);
            }
        });
        let stage_editor_options = StageEditorContextOptions {
            stage_number,
            object_id: None,
            local_tournament: self.local,
            origin_tournament: self.origin,
        };
        let stage_editor = StageEditorContext::new(stage_editor_options);
        self.stage_editors.update_value(|editors| {
            editors.insert(stage_number, stage_editor);
        });
    }

    pub fn get_stage_editor(&self, stage_number: u32) -> Option<StageEditorContext> {
        self.stage_editors
            .with_value(|editors| editors.get(&stage_number).copied())
    }

    pub fn prepare_group(&self, stage_number: u32, _group_number: u32) {
        self.prepare_stage(stage_number);
        // ToDo: implement group editor context and insert into map here
    }
}

#[derive(Clone, Copy)]
pub struct TournamentRefetchContext {
    /// Trigger to refetch data from server
    refetch_trigger: RwSignal<u64>,
    /// Read slice for getting the current state of the tournament editor
    pub track_fetch_trigger: Signal<u64>,
}

impl TournamentRefetchContext {
    pub fn new() -> Self {
        let refetch_trigger = RwSignal::new(0);
        Self {
            refetch_trigger,
            track_fetch_trigger: refetch_trigger.read_only().into(),
        }
    }

    pub fn trigger_refetch(&self) {
        self.refetch_trigger.update(|v| *v += 1);
    }
}
