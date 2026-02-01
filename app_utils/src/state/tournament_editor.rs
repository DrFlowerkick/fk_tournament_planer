//! Context for managing the tournament editor state.
//!
//! This module provides a context wrapper around `TournamentEditor` to ensure
//! efficient state updates via `RwSignal` without unnecessary cloning.

use crate::{
    error::{
        AppError,
        strategy::{handle_read_error, handle_write_error},
    },
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{GroupParams, SportParams, StageParams, TournamentBaseParams},
    server_fn::{
        stage::load_stage_by_number, tournament_base::load_tournament_base,
        tournament_editor::SaveTournamentEditorDiff,
    },
    state::{
        error_state::PageErrorContext,
        toast_state::{ToastContext, ToastVariant},
    },
};
use app_core::{
    TournamentEditor, TournamentEditorState, TournamentMode, TournamentState,
    utils::validation::ValidationResult,
};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_params, use_query},
};
use uuid::Uuid;

/****************************************************************************************
TODO: Refactoring TournamentEditorState
Option 1: Enable SSR by loading the complete tournament data in one go and initializing
          the TournamentEditor in one step. This would simplify state management
          but may increase initial load time.
          For this to work, we have to split EditTournament() into loading and editing
          components. The loading component would fetch the FULL tournament data (base +
          all stages/groups/etc., ideally via a single aggregated endpoint) and provide
          it as a prop to the editing component.
          The editing component then initializes the context with the provided data synchronously.
          The Resource for loading would move to the loading component.

Option 2: Keep current lazy loading approach but refactor TournamentEditorState
          to preload the origin tournament data when entering edit mode.
          The preload starts after fetching base. Using "spawn_local", we trigger
          background loading of stages/groups based on the base configuration.
          We have to handle race conditions: The user might start editing before preload
          finishes. Therefore, preloaded data updates 'origin', but 'local' is only
          updated if it hasn't been touched by the user yet.

Option 3: Hybrid "Loader Pattern". Split page components into a Loader (handling the Resource/SSR)
          and an Editor (presentation). The Loader passes fetched data via props to the Editor.
          The Editor renders immediately based on props (enabling real SSR for the content)
          and uses an Effect to sync these props into the global TournamentEditorContext.
          This avoids hydration errors because the parent context starts "empty" on both
          server and client, filling up only after hydration via the Effect.

Comparing options:
- Option 1 (Fetch-Then-Render): Best for consistency and SSR. No "partial" states logic needed.
  Requires a backend endpoint that dumps the full tournament structure.
- Option 2 (optimistic background loading): Faster initial render (TTFB) for large tournaments.
  More complex state synchronization logic (handling race conditions).
- Option 3 (Loader Pattern): Good balance. Allows granular SSR without massive initial payload.
  Ensures content is visible immediately via props, while context syncs slightly later for global UI.

Decide by KISS principle: Option 1 seems simpler architecturally (single source of truth),
despite the initial blocking load time. But Option requires less new code, therefore I start
with Option 3 for now.
*****************************************************************************************/

/// Context wrapper for `TournamentEditor`.
///
/// Uses an internal `RwSignal` to hold the state, encouraging the use of
/// `.update()` for mutations and `.with()` for reading to minimize cloning of heavy structures.
/// This context also provides various read/write slices for common properties
/// to facilitate fine-grained reactivity in the UI.
#[derive(Clone, Copy)]
pub struct TournamentEditorContext {
    // --- state & derived signals ---
    /// Tournament editor core state
    inner: RwSignal<TournamentEditor>,
    // ToDo: probably not required anymore
    /// Read slice for getting the state of tournament editor
    pub state: Signal<TournamentEditorState>,
    // ToDo: probably not required anymore
    /// Read slice for accessing the tournament base state, if any
    pub base_state: Signal<Option<TournamentState>>,
    /// Read slice for checking if we are creating a new tournament
    pub is_new_tournament: Signal<bool>,
    /// Read slice for checking if the editor is busy (saving/loading)
    pub is_busy: Signal<bool>,
    /// Read slice for checking if there are unsaved changes
    pub is_changed: Signal<bool>,
    /// Read slice for accessing the validation result of the tournament
    pub validation_result: Signal<ValidationResult<()>>,

    // --- URL Parameters & Queries ---
    /// Tournament ID for loading existing tournament bases
    pub tournament_id: Signal<Option<Uuid>>,
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
    pub set_base_num_entrants: Callback<u32>,
    /// Read slice for accessing the tournament base mode, if any
    pub base_mode: Signal<Option<TournamentMode>>,
    /// Write slice for setting the tournament base mode
    pub set_base_mode: Callback<TournamentMode>,
    /// Read slice for accessing the tournament base number of rounds for Swiss System, if any
    pub base_num_rounds_swiss_system: Signal<Option<u32>>,
    /// Write slice for setting the tournament base number of rounds for Swiss System
    pub set_base_num_rounds_swiss_system: Callback<u32>,
    /// Read slice for accessing the tournament  base state, if any
    pub base_tournament_state: Signal<Option<TournamentState>>,

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
    pub set_stage_num_groups: Callback<u32>,

    // --- Signals, Slices & Callbacks for Current Group ---
    /// Read slice for checking if stage is initialized
    pub is_group_initialized: Signal<bool>,
}

impl Default for TournamentEditorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TournamentEditorContext {
    /// Creates a new, empty context.
    pub fn new() -> Self {
        // --- navigation and globale state context ---
        let navigate = use_navigate();
        let UseQueryNavigationReturn {
            url_with_update_query,
            url_route_with_sub_path,
            path,
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
        let inner = RwSignal::new(TournamentEditor::new());
        let state = create_read_slice(inner, |inner| inner.get_state());
        let base_state = create_read_slice(inner, |inner| {
            inner.get_base().map(|b| b.get_tournament_state())
        });
        let is_changed = create_read_slice(inner, |inner| inner.is_changed());
        let validation_result = create_read_slice(inner, |inner| inner.validation());

        // --- url parameters & queries & validation ---
        let sport_id_query = use_query::<SportParams>();
        let sport_id = Signal::derive(move || {
            sport_id_query.with(|q| q.as_ref().ok().and_then(|p| p.sport_id))
        });
        let tournament_id_query = use_query::<TournamentBaseParams>();
        let tournament_id = Signal::derive(move || {
            tournament_id_query.with(|q| q.as_ref().ok().and_then(|p| p.tournament_id))
        });
        let is_new_tournament = Signal::derive(move || {
            tournament_id.get().is_none() && path.get().starts_with("/new-tournament")
        });
        let stage_params = use_params::<StageParams>();
        let active_stage_number = Signal::derive(move || {
            stage_params.with(|p| p.as_ref().ok().and_then(|params| params.stage_number))
        });
        let group_params = use_params::<GroupParams>();
        let active_group_number = Signal::derive(move || {
            group_params.with(|p| p.as_ref().ok().and_then(|params| params.group_number))
        });

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
                    navigate(
                        &url_route_with_sub_path(&redirect_path),
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
        let base_res = LocalResource::new(move || {
            let maybe_t_id = tournament_id.get();
            let maybe_s_id = sport_id.get();
            // create future for loading tournament base
            async move {
                if let Some(t_id) = maybe_t_id
                    && maybe_s_id.is_some()
                {
                    match load_tournament_base(t_id).await {
                        Ok(None) => Err(AppError::ResourceNotFound(
                            "Tournament Base".to_string(),
                            t_id,
                        )),
                        load_result => load_result,
                    }
                } else {
                    Ok(None)
                }
            }
        });
        let stage_res = LocalResource::new(move || {
            let maybe_t_id = tournament_id.get();
            let maybe_s_num = active_stage_number.get();
            // create future for loading stage
            async move {
                if let Some(t_id) = maybe_t_id
                    && let Some(stage_number) = maybe_s_num
                {
                    load_stage_by_number(t_id, stage_number).await
                } else {
                    Ok(None)
                }
            }
        });

        // server action & resource activity tracking
        let is_busy = Signal::derive(move || {
            save_diff.pending().get() || base_res.get().is_none() || stage_res.get().is_none()
        });

        // --- effects for server action and resource results ---
        // retry function for error handling
        let refetch_and_reset = Callback::new(move |()| {
            save_diff.clear();
            base_res.refetch();
            stage_res.refetch();
        });

        // cancel function for cancel button and error handling
        let on_cancel = use_on_cancel();

        // Effect to handle save action results
        Effect::new({
            let navigate = navigate.clone();
            move || match save_diff.value().get() {
                Some(Ok(base_id)) => {
                    toast_ctx.add("Tournament saved successfully", ToastVariant::Success);

                    if tournament_id.get().is_some() {
                        // if it was a new tournament, trigger refetch to load the full data
                        refetch_and_reset.run(());
                    } else {
                        // clear save action state
                        save_diff.clear();
                        // else navigate directly
                        let nav_url =
                            url_with_update_query("tournament_id", &base_id.to_string(), None);
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

        // Effect to load tournament base when base resource changes
        Effect::new(move || match base_res.get() {
            Some(Ok(Some(base))) => {
                inner.update(|te| {
                    te.set_base(base);
                });
            }
            Some(Ok(None)) => {
                // initialize or reset base, if we have no tournament id in query and a sport id is given
                // and we are in None or Edit state -> create new base
                if let Some(s_id) = sport_id.get()
                    && tournament_id.get().is_none()
                    && matches!(
                        inner.get_untracked().get_state(),
                        TournamentEditorState::None | TournamentEditorState::Edit
                    )
                {
                    inner.update(move |te| {
                        te.new_base(s_id);
                    });
                }
            }
            Some(Err(err)) => {
                handle_read_error(
                    &page_err_ctx,
                    component_id.get_value(),
                    &err,
                    refetch_and_reset,
                    on_cancel,
                );
            }
            None => { /* loading state - do nothing */ }
        });

        // Effect to load current stage when stage resource changes
        Effect::new(move || match stage_res.get() {
            Some(Ok(Some(stage))) => {
                inner.update(|te| {
                    te.set_stage(stage);
                });
            }
            Some(Ok(None)) => {
                // create new stage, if we have a valid stage number in params, but no such stage exists yet
                if let Some(stage_number) = active_stage_number.get()
                    && let Some(number_of_stages) = inner
                        .get_untracked()
                        .get_base()
                        .map(|b| b.get_tournament_mode().get_num_of_stages())
                    && stage_number < number_of_stages
                    && inner
                        .get_untracked()
                        .get()
                        .get_stage_by_number(stage_number)
                        .is_none()
                {
                    inner.update(move |te| {
                        te.new_stage(stage_number);
                    });
                }
            }
            Some(Err(err)) => {
                handle_read_error(
                    &page_err_ctx,
                    component_id.get_value(),
                    &err,
                    refetch_and_reset,
                    on_cancel,
                );
            }
            None => { /* loading state - do nothing */ }
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
        let set_base_num_entrants = Callback::new(move |num_entrants: u32| {
            set_base_num_entrants.set(num_entrants);
        });
        let (base_mode, set_base_mode) = create_slice(
            inner,
            |inner| inner.get_base().map(|b| b.get_tournament_mode()),
            |inner, mode: TournamentMode| {
                inner.get_local_mut().set_base_mode(mode);
            },
        );
        let set_base_mode = Callback::new(move |mode: TournamentMode| {
            set_base_mode.set(mode);
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
        let set_base_num_rounds_swiss_system = Callback::new(move |num_rounds_swiss: u32| {
            set_base_num_rounds_swiss_system.set(num_rounds_swiss);
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
        let set_stage_num_groups = Callback::new(move |num_groups: u32| {
            set_stage_num_groups.set(num_groups);
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
            state,
            base_state,
            is_new_tournament,
            is_busy,
            is_changed,
            validation_result,
            // url parameters & queries
            tournament_id,
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
            base_tournament_state,
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
