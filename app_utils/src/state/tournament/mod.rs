//! Context for managing the tournament editor state.
//!
//! This module provides a context wrapper around `TournamentEditor` to ensure
//! efficient state updates via `RwSignal` without unnecessary cloning.

pub mod base;

use crate::{
    error::strategy::handle_write_error,
    hooks::use_query_navigation::{
        MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
    },
    params::{GroupNumberParams, ParamQuery, StageNumberParams, TournamentBaseIdQuery},
    server_fn::tournament_editor::SaveTournamentEditorDiff,
    state::{
        EditorContext, activity_tracker::ActivityTracker, error_state::PageErrorContext,
        toast_state::ToastContext,
    },
};
use app_core::{
    Stage, Tournament, TournamentBase, TournamentMode, TournamentState,
    utils::{
        id_version::IdVersion,
        validation::{FieldError, ValidationResult},
    },
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use uuid::Uuid;

/// This context provides efficient access to the tournament editor state
/// via `RwSignal`, along with various read slices and callbacks for
/// interacting with the editor.
/// It also manages server actions for saving changes and handles
/// navigation and error handling.
#[derive(Clone, Copy)]
pub struct TournamentEditorContext {
    // --- state & derived signals ---
    /// The local editable tournament
    local: RwSignal<Option<Tournament>>,
    /// The original tournament loaded from storage.
    origin: RwSignal<Option<Tournament>>,
    /// Read slice of origin
    pub origin_read_only: Signal<Option<Tournament>>,
    /// Read slice for accessing the validation result of the tournament
    pub validation_result: Signal<ValidationResult<()>>,
    /// WriteSignal for setting a unique violation error on the name field, if any
    pub set_unique_violation_error: WriteSignal<Option<FieldError>>,

    // --- Actions and Resources ---
    /// Action for saving tournament editor diffs
    save_diff: ServerAction<SaveTournamentEditorDiff>,

    // --- Optimistic version handling for tournament ---
    /// Signal for optimistic version handling to prevent unneeded server round after save
    pub optimistic_version: Signal<Option<u32>>,
    /// WriteSignal for optimistic version handling to prevent unneeded server round after save
    set_optimistic_version: RwSignal<Option<u32>>,

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
    pub set_base_name: Callback<Option<String>>,
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
    /// Currently active stage number from URL, if any
    pub active_stage_number: Signal<Option<u32>>,
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
    /// Currently active group number from URL, if any
    pub active_group_number: Signal<Option<u32>>,
    /// Read slice for checking if stage is initialized
    pub is_group_initialized: Signal<bool>,
}

impl EditorContext for TournamentEditorContext {
    type ObjectType = Tournament;
    type NewEditorOptions = ();

    /// Creates a new, empty context.
    fn new(_: ()) -> Self {
        // --- refetch context ---
        // ToDo: we probably do not need this anymore
        let refetch_trigger = expect_context::<TournamentRefetchContext>();

        // --- navigation and globale state context ---
        let navigate = use_navigate();
        let UseQueryNavigationReturn {
            url_update_query,
            url_matched_route,
            ..
        } = use_query_navigation();
        let page_err_ctx = expect_context::<PageErrorContext>();
        let toast_ctx = expect_context::<ToastContext>();
        let component_id = StoredValue::new(Uuid::new_v4());
        let activity_tracker = expect_context::<ActivityTracker>();
        // remove errors on unmount
        on_cleanup(move || {
            page_err_ctx.clear_all_for_component(component_id.get_value());
            activity_tracker.remove_component(component_id.get_value());
        });

        // --- core signals ---
        let local = RwSignal::new(None::<Tournament>);
        let origin = RwSignal::new(None::<Tournament>);
        let base_state = create_read_slice(local, |local| {
            local.as_ref().map(|t| t.get_base().get_tournament_state())
        });
        let (unique_violation_error, set_unique_violation_error) = signal(None::<FieldError>);
        let validation_result = Signal::derive(move || {
            let vr = local.with(|local| {
                if let Some(t) = local {
                    t.validate()
                } else {
                    ValidationResult::Ok(())
                }
            });
            if let Some(unique_err) = unique_violation_error.get() {
                if let Err(mut validation_errors) = vr {
                    validation_errors.add(unique_err);
                    Err(validation_errors)
                } else {
                    Err(unique_err.into())
                }
            } else {
                vr
            }
        });
        let set_optimistic_version = RwSignal::new(None);

        // --- url parameters & queries & validation ---
        let tournament_id = TournamentBaseIdQuery::use_param_query();
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
        let save_diff_pending = save_diff.pending();
        activity_tracker.track_pending_memo(component_id.get_value(), save_diff_pending);

        // --- effects for server action and resource results ---
        // retry function for error handling
        let refetch = Callback::new(move |()| {
            refetch_trigger.trigger_refetch();
        });

        // Effect to handle save action results
        Effect::new({
            let navigate = navigate.clone();
            move || match save_diff.value().get() {
                Some(Ok(base_id)) => {
                    toast_ctx.success("Tournament saved successfully");
                    // clear save action state
                    save_diff.clear();
                    if tournament_id.get().is_some() {
                        // if it was an existing tournament, trigger refetch to load the full data
                        refetch.run(());
                    } else {
                        // else navigate directly
                        let nav_url = url_update_query("tournament_id", &base_id.to_string());
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
                        refetch,
                    );
                }
                None => { /* saving state - do nothing */ }
            }
        });

        // --- Create slices for base ---
        let is_base_initialized = create_read_slice(local, |local| local.is_some());
        let base_tournament_state = create_read_slice(local, |local| {
            local.as_ref().map(|t| t.get_base().get_tournament_state())
        });
        let is_disabled_base_editing = Signal::derive(move || {
            matches!(
                base_tournament_state.get(),
                Some(TournamentState::ActiveStage(_)) | Some(TournamentState::Finished)
            )
        });
        let base_id =
            create_read_slice(local, |local| local.as_ref().map(|t| t.get_base().get_id()));
        let (base_name, set_base_name) = create_slice(
            local,
            |local| local.as_ref().map(|t| t.get_base().get_name().to_string()),
            |local, name: String| {
                if let Some(t) = local {
                    t.set_base_name(name);
                }
            },
        );
        let set_base_name = Callback::new(move |name: Option<String>| {
            set_base_name.set(name.unwrap_or_default());
        });
        let (base_num_entrants, set_base_num_entrants) = create_slice(
            local,
            |local| local.as_ref().map(|t| t.get_base().get_num_entrants()),
            |local, num_entrants: u32| {
                if let Some(t) = local {
                    t.set_base_num_entrants(num_entrants);
                }
            },
        );
        let set_base_num_entrants = Callback::new(move |num_entrants: Option<u32>| {
            set_base_num_entrants.set(num_entrants.unwrap_or_default());
        });
        let (base_mode, set_base_mode) = create_slice(
            local,
            |local| local.as_ref().map(|t| t.get_base().get_tournament_mode()),
            |local, mode: TournamentMode| {
                if let Some(t) = local {
                    t.set_base_mode(mode);
                }
            },
        );
        let set_base_mode = Callback::new(move |mode: Option<TournamentMode>| {
            if let Some(mode) = mode {
                set_base_mode.set(mode);
            }
        });
        let (base_num_rounds_swiss_system, set_base_num_rounds_swiss_system) = create_slice(
            local,
            |local| {
                local
                    .as_ref()
                    .and_then(|t| t.get_base().get_num_rounds_swiss_system())
            },
            |local, num_rounds_swiss: u32| {
                if let Some(t) = local {
                    t.set_base_num_rounds_swiss_system(num_rounds_swiss);
                }
            },
        );
        let set_base_num_rounds_swiss_system =
            Callback::new(move |num_rounds_swiss: Option<u32>| {
                set_base_num_rounds_swiss_system.set(num_rounds_swiss.unwrap_or_default());
            });

        // --- Create slices for stage ---
        let is_stage_initialized = create_read_slice(local, move |local| {
            if let Some(sn) = active_stage_number.get() {
                local
                    .as_ref()
                    .and_then(|t| t.get_stage_by_number(sn))
                    .is_some()
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
        let active_stage_id = create_read_slice(local, move |local| {
            active_stage_number
                .get()
                .and_then(|sn| local.as_ref().and_then(|t| t.get_stage_by_number(sn)))
                .map(|stage| stage.get_id())
        });
        let (stage_num_groups, set_stage_num_groups) = create_slice(
            local,
            move |local| {
                active_stage_number
                    .get()
                    .and_then(|sn| local.as_ref().and_then(|t| t.get_stage_by_number(sn)))
                    .map(|stage| stage.get_num_groups())
            },
            move |local, num_groups: u32| {
                if let Some(id) = active_stage_id.get()
                    && let Some(t) = local.as_mut()
                {
                    t.set_stage_number_of_groups(id, num_groups);
                }
            },
        );
        let set_stage_num_groups = Callback::new(move |num_groups: Option<u32>| {
            set_stage_num_groups.set(num_groups.unwrap_or_default());
        });

        // --- Create slices for group ---
        let is_group_initialized = create_read_slice(local, move |local| {
            if let Some(sn) = active_stage_number.get()
                && let Some(gn) = active_group_number.get()
                && let Some(t) = local.as_ref()
            {
                t.get_group_by_number(sn, gn).is_some()
            } else {
                false
            }
        });

        Self {
            // core signals
            local,
            origin,
            origin_read_only: origin.into(),
            validation_result,
            set_unique_violation_error,
            optimistic_version: set_optimistic_version.into(),
            set_optimistic_version,
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
            active_stage_number,
            is_stage_initialized,
            is_hiding_stage_editor,
            is_disabled_stage_editing,
            active_stage_id,
            stage_num_groups,
            set_stage_num_groups,
            // group slices
            active_group_number,
            is_group_initialized,
        }
    }

    /// Get the original tournament currently loaded in the editor context, if any.
    fn get_origin(&self) -> Option<Tournament> {
        self.origin
            .with(|editor| editor.as_ref().map(|t| t.clone()))
    }

    /// Set the current tournament in the editor context, updating all relevant state accordingly.
    fn set_object(&self, tournament: Tournament) {
        self.set_optimistic_version
            .set(tournament.get_base().get_version());
        self.local.set(Some(tournament.clone()));
        self.origin.set(Some(tournament));
    }

    /// Create a new tournament object in the editor context, returning its unique identifier.
    fn new_object(&self) -> Option<Uuid> {
        let tournament = Tournament::new();
        let id = tournament.get_base().get_id();
        self.set_optimistic_version.set(None);
        self.local.set(Some(tournament.clone()));
        self.origin.set(None);
        Some(id)
    }

    /// Increment the optimistic version in the editor context to optimistically handle version updates after saving.
    fn increment_optimistic_version(&self) {
        self.set_optimistic_version.update(|v| {
            if let Some(current_version) = v {
                *current_version += 1
            } else {
                *v = Some(0)
            }
        });
    }

    /// If save fails, we need to reset the version to the original version to prevent version mismatch on next save attempt.
    fn reset_version_to_origin(&self) {
        let origin_version = self
            .origin
            .with(|origin| origin.as_ref().and_then(|t| t.get_base().get_version()));
        self.set_optimistic_version.set(origin_version);
    }

    /// Get the current optimistic version signal from the editor context, if any.
    fn get_optimistic_version(&self) -> Signal<Option<u32>> {
        self.optimistic_version
    }
}

impl TournamentEditorContext {
    pub fn new_stage(&self, stage_number: u32) {
        self.local.update(|te| {
            if let Some(t) = te.as_mut() {
                t.new_stage(stage_number);
            }
        });
    }

    pub fn set_stage(&self, stage: Stage) {
        self.local.update(|te| {
            if let Some(t) = te.as_mut() {
                t.set_stage(stage);
            }
        });
    }

    // Save diff
    pub fn save_diff(&self) {
        /*if let Some(base_id) = self.base_id.get() {
            self.inner.with_untracked(|te| {
                self.save_diff.dispatch(SaveTournamentEditorDiff {
                    base_id,
                    base_diff: te.collect_base_diff().cloned(),
                    stages_diff: te.collect_stages_diff(),
                });
            })
        }*/
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
