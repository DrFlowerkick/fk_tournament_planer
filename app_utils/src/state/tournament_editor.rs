//! Context for managing the tournament editor state.
//!
//! This module provides a context wrapper around `TournamentEditor` to ensure
//! efficient state updates via `RwSignal` without unnecessary cloning.

use crate::{
    error::strategy::handle_write_error,
    hooks::use_query_navigation::{
        MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
    },
    params::{use_group_number_params, use_stage_number_params, use_tournament_base_id_query},
    server_fn::tournament_editor::SaveTournamentEditorDiff,
    state::{
        error_state::PageErrorContext,
        toast_state::{ToastContext, ToastVariant},
    },
};
use app_core::{
    Stage, TournamentEditor, TournamentMode, TournamentState, utils::validation::ValidationResult,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

/// Context wrapper for `TournamentEditor`.
///
/// This context provides efficient access to the tournament editor state
/// via `RwSignal`, along with various read slices and callbacks for
/// interacting with the editor.
/// It also manages server actions for saving changes and handles
/// navigation and error handling.
#[derive(Clone, Copy)]
pub struct TournamentEditorContext {
    // --- state & derived signals ---
    /// Tournament editor core state
    inner: RwSignal<TournamentEditor>,
    /// Read slice for checking if the editor is busy (saving/loading)
    pub is_busy: Signal<bool>,
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the tournament
    pub validation_result: Signal<ValidationResult<()>>,
    /// Currently active stage number from URL, if any
    pub active_stage_number: Signal<Option<u32>>,

    // --- Actions and Resources ---
    /// Action for saving tournament editor diffs
    save_diff: ServerAction<SaveTournamentEditorDiff>,

    // --- Signals, Slices & Callbacks for Tournament Base ---
    /// Read slice for checking if base is initialized
    pub is_base_initialized: Signal<bool>,
    /// Read slice for checking if base editing is disabled
    pub is_disabled_base_editing: Signal<bool>,
    /// Read slice for accessing the tournament base ID, if any
    pub base_id: Signal<Option<Uuid>>,
    /// Read slice for accessing the tournament base name, if any
    pub base_name: Signal<Option<String>>,
    /// Write slice for setting the tournament base name
    pub set_base_name: Callback<String>,
    /// Read slice for accessing the tournament base number of entrants, if any
    pub base_num_entrants: Signal<Option<u32>>,
    /// Write slice for setting the tournament base number of entrants
    pub set_base_num_entrants: Callback<Option<u32>>,
    /// Read slice for accessing the tournament base mode, if any
    pub base_mode: Signal<Option<TournamentMode>>,
    /// Write slice for setting the tournament base mode
    pub set_base_mode: Callback<Option<TournamentMode>>,
    /// Read slice for accessing the tournament base number of rounds for Swiss System, if any
    pub base_num_rounds_swiss_system: Signal<Option<u32>>,
    /// Write slice for setting the tournament base number of rounds for Swiss System
    pub set_base_num_rounds_swiss_system: Callback<Option<u32>>,

    // --- Signals, Slices & Callbacks for Current Stage ---
    /// Read slice for checking if stage is initialized
    pub is_stage_initialized: Signal<bool>,
    /// Read slice for checking if stage editor is hidden
    pub is_hiding_stage_editor: Signal<bool>,
    /// Read slice for checking if stage editing is disabled
    pub is_disabled_stage_editing: Signal<bool>,
    /// Read slice for accessing the active stage ID, if any
    pub active_stage_id: Signal<Option<Uuid>>,
    /// Read slice for accessing the current stage number of groups, if any
    pub stage_num_groups: Signal<Option<u32>>,
    /// Write slice for setting the current stage number of groups
    pub set_stage_num_groups: Callback<Option<u32>>,

    // --- Signals, Slices & Callbacks for Current Group ---
    /// Read slice for checking if stage is initialized
    pub is_group_initialized: Signal<bool>,
}

impl TournamentEditorContext {
    /// Creates a new, empty context.
    pub fn new(initialized_tournament_editor: TournamentEditor) -> Self {
        leptos::logging::log!(
            "Initializing TournamentEditorContext for tournament editor"
        );
        let refetch_trigger = expect_context::<TournamentRefetchContext>();

        // --- navigation and globale state context ---
        let navigate = use_navigate();
        let UseQueryNavigationReturn {
            url_matched_route_update_query,
            url_matched_route,
            ..
        } = use_query_navigation();
        let page_err_ctx = expect_context::<PageErrorContext>();
        let toast_ctx = expect_context::<ToastContext>();
        let component_id = StoredValue::new(Uuid::new_v4());
        // remove errors on unmount
        on_cleanup(move || {
            page_err_ctx.clear_all_for_component(component_id.get_value());
        });

        // --- core signals ---
        let inner = RwSignal::new(initialized_tournament_editor);
        let base_state = create_read_slice(inner, |inner| {
            inner.get_base().map(|b| b.get_tournament_state())
        });
        let is_changed = create_read_slice(inner, |inner| inner.is_changed());
        let validation_result = create_read_slice(inner, |inner| inner.validation());

        // --- url parameters & queries & validation ---
        let tournament_id = use_tournament_base_id_query();
        let active_stage_number = use_stage_number_params();
        let active_group_number = use_group_number_params();

        let valid_object_numbers = Memo::new(move |_| {
            inner.with(|state| {
                state.validate_object_numbers(
                    active_stage_number.get(),
                    active_group_number.get(),
                    None,
                    None,
                )
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
                    let url = url_matched_route(MatchedRouteHandler::Extend(&redirect_path));
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

        // --- server actions and resources ---
        let save_diff = ServerAction::<SaveTournamentEditorDiff>::new();

        // server action & resource activity tracking
        let is_busy = Signal::derive(move || save_diff.pending().get());

        // --- effects for server action and resource results ---
        // retry function for error handling
        let refetch_and_reset = Callback::new(move |()| {
            refetch_trigger.trigger_refetch();
            save_diff.clear();
        });

        // Effect to handle save action results
        Effect::new({
            let navigate = navigate.clone();
            move || match save_diff.value().get() {
                Some(Ok(base_id)) => {
                    toast_ctx.add("Tournament saved successfully", ToastVariant::Success);
                    // clear save action state
                    save_diff.clear();
                    if tournament_id.get().is_some() {
                        // if it was an existing tournament, trigger refetch to load the full data
                        refetch_and_reset.run(());
                    } else {
                        // else navigate directly
                        let nav_url = url_matched_route_update_query(
                            "tournament_id",
                            &base_id.to_string(),
                            MatchedRouteHandler::Keep,
                        );
                        navigate(
                            &nav_url,
                            NavigateOptions {
                                replace: true,
                                scroll: false,
                                ..Default::default()
                            },
                        );
                    }
                }
                Some(Err(err)) => {
                    leptos::logging::error!("Error saving tournament editor: {:?}", err);
                    save_diff.clear();
                    handle_write_error(
                        &page_err_ctx,
                        &toast_ctx,
                        component_id.get_value(),
                        &err,
                        refetch_and_reset,
                    );
                }
                None => { /* saving state - do nothing */ }
            }
        });

        // --- Create slices for base ---
        let is_base_initialized = create_read_slice(inner, |inner| inner.get_base().is_some());
        let base_tournament_state = create_read_slice(inner, |inner| {
            inner.get_base().map(|b| b.get_tournament_state())
        });
        let is_disabled_base_editing = Signal::derive(move || {
            matches!(
                base_tournament_state.get(),
                Some(TournamentState::ActiveStage(_)) | Some(TournamentState::Finished)
            )
        });
        let base_id = create_read_slice(inner, |inner| inner.get_base().map(|b| b.get_id()));
        let (base_name, set_base_name) = create_slice(
            inner,
            |inner| inner.get_base().map(|b| b.get_name().to_string()),
            |inner, name: String| {
                inner.get_local_mut().set_base_name(name);
            },
        );
        let set_base_name = Callback::new(move |name: String| {
            set_base_name.set(name);
        });
        let (base_num_entrants, set_base_num_entrants) = create_slice(
            inner,
            |inner| inner.get_base().map(|b| b.get_num_entrants()),
            |inner, num_entrants: u32| {
                inner.get_local_mut().set_base_num_entrants(num_entrants);
            },
        );
        let set_base_num_entrants = Callback::new(move |num_entrants: Option<u32>| {
            set_base_num_entrants.set(num_entrants.unwrap_or_default());
        });
        let (base_mode, set_base_mode) = create_slice(
            inner,
            |inner| inner.get_base().map(|b| b.get_tournament_mode()),
            |inner, mode: TournamentMode| {
                inner.get_local_mut().set_base_mode(mode);
            },
        );
        let set_base_mode = Callback::new(move |mode: Option<TournamentMode>| {
            if let Some(mode) = mode {
                set_base_mode.set(mode);
            }
        });
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
        let set_base_num_rounds_swiss_system =
            Callback::new(move |num_rounds_swiss: Option<u32>| {
                set_base_num_rounds_swiss_system.set(num_rounds_swiss.unwrap_or_default());
            });

        // --- Create slices for stage ---
        let is_stage_initialized = create_read_slice(inner, move |inner| {
            if let Some(sn) = active_stage_number.get() {
                inner.get().get_stage_by_number(sn).is_some()
            } else {
                false
            }
        });
        let is_hiding_stage_editor = Signal::derive(move || {
            matches!(
                base_mode.get(),
                Some(TournamentMode::SingleStage)
                    | Some(TournamentMode::SwissSystem { num_rounds: _ })
            )
        });
        let is_disabled_stage_editing = Signal::derive(move || {
            if let Some(sn) = active_stage_number.get()
                && let Some(tournament_state) = base_state.get()
            {
                match tournament_state {
                    TournamentState::ActiveStage(acs) => acs >= sn,
                    TournamentState::Finished => true,
                    _ => false,
                }
            } else {
                false
            }
        });
        let active_stage_id = create_read_slice(inner, |inner| inner.get_active_stage_id());
        let (stage_num_groups, set_stage_num_groups) = create_slice(
            inner,
            |inner| inner.get_active_stage().map(|s| s.get_num_groups()),
            |inner, num_groups: u32| {
                if let Some(stage_id) = inner.get_active_stage_id() {
                    inner
                        .get_local_mut()
                        .set_stage_number_of_groups(stage_id, num_groups);
                }
            },
        );
        let set_stage_num_groups = Callback::new(move |num_groups: Option<u32>| {
            set_stage_num_groups.set(num_groups.unwrap_or_default());
        });

        // --- Create slices for group ---
        let is_group_initialized = create_read_slice(inner, move |inner| {
            if let Some(sn) = active_stage_number.get()
                && let Some(gn) = active_group_number.get()
            {
                inner.get().get_group_by_number(sn, gn).is_some()
            } else {
                false
            }
        });

        Self {
            // core signals
            inner,
            is_busy,
            is_changed,
            validation_result,
            // url parameters & queries
            active_stage_number,
            // actions and resources
            save_diff,
            // base slices
            is_base_initialized,
            is_disabled_base_editing,
            base_id,
            base_name,
            set_base_name,
            base_num_entrants,
            set_base_num_entrants,
            base_mode,
            set_base_mode,
            base_num_rounds_swiss_system,
            set_base_num_rounds_swiss_system,
            // stage slices
            is_stage_initialized,
            is_hiding_stage_editor,
            is_disabled_stage_editing,
            active_stage_id,
            stage_num_groups,
            set_stage_num_groups,
            // group slices
            is_group_initialized,
        }
    }

    pub fn new_stage(&self, stage_number: u32) {
        self.inner.update(|te| {
            te.new_stage(stage_number);
        });
    }

    pub fn set_stage(&self, stage: Stage) {
        self.inner.update(|te| {
            te.set_stage(stage);
        });
    }

    // Save diff
    pub fn save_diff(&self) {
        if let Some(base_id) = self.base_id.get() {
            self.inner.with_untracked(|te| {
                self.save_diff.dispatch(SaveTournamentEditorDiff {
                    base_id,
                    base_diff: te.collect_base_diff().cloned(),
                    stages_diff: te.collect_stages_diff(),
                });
            })
        }
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
