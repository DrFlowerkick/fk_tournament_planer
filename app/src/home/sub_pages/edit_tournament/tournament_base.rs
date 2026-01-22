//! create or edit a tournament

use app_core::{
    TournamentBase, TournamentMode, TournamentState, TournamentType,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use app_utils::{
    components::inputs::{EnumSelect, ValidatedNumberInput, ValidatedTextInput},
    error::{
        AppError,
        strategy::{handle_read_error, handle_write_error},
    },
    hooks::{
        is_field_valid::is_field_valid,
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{SportParams, StageParams, TournamentBaseParams},
    server_fn::{
        stage::SaveStage,
        tournament_base::{SaveTournamentBase, load_tournament_base},
    },
    state::{
        error_state::PageErrorContext,
        toast_state::{ToastContext, ToastVariant},
        tournament_editor::context::TournamentEditorContext,
    },
};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    components::A,
    hooks::{use_navigate, use_params, use_query},
    nested_router::Outlet,
};
use uuid::Uuid;

#[component]
pub fn EditTournament() -> impl IntoView {
    // --- Initialize context for creating and editing tournaments ---
    let tournament_editor_context = TournamentEditorContext::new();
    provide_context(tournament_editor_context);
    let page_err_ctx = expect_context::<PageErrorContext>();
    let toast_ctx = expect_context::<ToastContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    let is_changed = move || tournament_editor_context.is_changed();

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_route_with_sub_path,
        url_with_update_query,
        path,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    let sport_id_query = use_query::<SportParams>();
    let tournament_id_query = use_query::<TournamentBaseParams>();

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    // None = effectively navigating away or invalid state.
    let sport_id = move || sport_id_query.get().ok().and_then(|p| p.sport_id);

    let tournament_id = move || tournament_id_query.get().ok().and_then(|p| p.tournament_id);

    let is_new = Signal::derive(move || {
        tournament_id().is_none() && path.get().starts_with("/new-tournament")
    });

    // --- Structural Integrity & Auto-Navigation Watchdog ---
    // params in url
    // ToDo: add params for objects, which are not implemented, yet.
    let stage_params = use_params::<StageParams>();
    let stage_number = move || {
        stage_params
            .get_untracked()
            .ok()
            .and_then(|p| p.stage_number)
    };

    // url validation effect
    Effect::new({
        let navigate = navigate.clone();
        move || {
            // Listen to the explicit trigger
            tournament_editor_context.url_validation_trigger().track();

            // Validate url against current params and navigate if invalid params detected
            if let Some(redirect_path) =
                tournament_editor_context.validate_url(stage_number(), None, None, None)
            {
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

    // --- Form Signals ---
    let set_id_version = RwSignal::new(IdVersion::New);
    let set_name = RwSignal::new("".to_string());
    let set_entrants = RwSignal::new(0_u32);
    let t_type = RwSignal::new(TournamentType::Scheduled);
    let set_mode = RwSignal::new(TournamentMode::SingleStage);
    let set_num_rounds_swiss = RwSignal::new(0_u32);
    let state = RwSignal::new(TournamentState::Draft);

    let set_signals_from_tournament = move |tournament: &TournamentBase| {
        set_id_version.set(tournament.get_id_version());
        set_name.set(tournament.get_name().to_string());
        set_entrants.set(tournament.get_num_entrants());
        t_type.set(tournament.get_tournament_type());
        set_mode.set(tournament.get_tournament_mode());
        if let TournamentMode::SwissSystem { num_rounds } = tournament.get_tournament_mode() {
            set_num_rounds_swiss.set(num_rounds);
        }
        state.set(tournament.get_tournament_state());
    };

    // --- Server Resources & Actions  ---
    // load tournament base resource
    let tournament_res = Resource::new(
        move || (tournament_id(), sport_id()),
        move |(maybe_t_id, maybe_s_id)| async move {
            // do we really need to check sport_id here?
            if let Some(_s_id) = maybe_s_id {
                if let Some(t_id) = maybe_t_id {
                    return match load_tournament_base(t_id).await {
                        Ok(None) => Err(AppError::ResourceNotFound(
                            "Tournament Base".to_string(),
                            t_id,
                        )),
                        load_result => load_result,
                    };
                }
            }
            Ok(None)
        },
    );

    // save tournament base action
    let save_tournament_base = ServerAction::<SaveTournamentBase>::new();

    // save stage action
    let save_stage = ServerAction::<SaveStage>::new();

    // retry function for error handling
    let refetch_and_reset = move || {
        save_tournament_base.clear();
        save_stage.clear();
        tournament_res.refetch();
    };

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // handle load tournament base results
    Effect::new({
        let on_cancel = on_cancel.clone();
        move || {
            match tournament_res.get() {
                Some(Ok(Some(tournament))) => {
                    // successful load from database
                    set_signals_from_tournament(&tournament);
                    tournament_editor_context.set_tournament(tournament.clone(), !is_new.get());
                }
                Some(Ok(None)) => {
                    // Case A: Tournament was saved and tournament_id is set in url query,
                    // but resource status has not changed yet (still loading).
                    if let Some(t_id) = tournament_id()
                        && let Some(origin) =
                            tournament_editor_context.get_origin_tournament_untracked()
                        && origin.get_id() == Some(t_id)
                    {
                        // wait for resource to load
                        return;
                    }
                    // Case B: New Tournament Mode (No ID in URL)
                    // Check if we need to enforce a reset (Force New).
                    // Reset is needed if:
                    // 1. Context is empty (None)
                    // OR
                    // 2. The origin tournament currently in context has a real ID (Saved Tournament).
                    //    This handles the switch from "Edit A" -> "New B" without unmounting.
                    if let Some(s_id) = sport_id()
                        && (tournament_editor_context
                            .get_tournament_untracked()
                            .is_none()
                            || tournament_editor_context
                                .get_origin_tournament_untracked()
                                .is_some())
                    {
                        let mut tournament = TournamentBase::default();
                        tournament
                            .set_sport_id(s_id)
                            .set_id_version(IdVersion::NewWithId(Uuid::new_v4()));
                        set_signals_from_tournament(&tournament);
                        tournament_editor_context.set_tournament(tournament, false);
                    }
                }
                Some(Err(err)) => {
                    handle_read_error(
                        &page_err_ctx,
                        component_id.get_value(),
                        &err,
                        refetch_and_reset.clone(),
                        on_cancel.clone(),
                    );
                }
                None => { /* loading state - do nothing */ }
            }
        }
    });

    // handle save tournament base results
    Effect::new({
        let navigate = navigate.clone();
        move || match save_tournament_base.value().get() {
            Some(Ok(tb)) => {
                save_tournament_base.clear();
                toast_ctx.add("Tournament saved successfully", ToastVariant::Success);
                let nav_url = url_with_update_query(
                    "tournament_id",
                    &tb.get_id().map(|id| id.to_string()).unwrap_or_default(),
                    None,
                );
                // set saved tb as origin in editor context before navigation to prevent
                // load effect to reset the editor state
                tournament_editor_context.set_tournament(tb, true);
                navigate(
                    &nav_url,
                    NavigateOptions {
                        replace: true,
                        scroll: false,
                        ..Default::default()
                    },
                );
            }
            Some(Err(err)) => {
                handle_write_error(
                    &page_err_ctx,
                    &toast_ctx,
                    component_id.get_value(),
                    &err,
                    refetch_and_reset.clone(),
                );
            }
            None => { /* saving state - do nothing */ }
        }
    });

    // handle save stage results
    Effect::new(move || match save_stage.value().get() {
        Some(Ok(_stage)) => {
            save_stage.clear();
            toast_ctx.add("Stage saved successfully", ToastVariant::Success);
        }
        Some(Err(err)) => {
            handle_write_error(
                &page_err_ctx,
                &toast_ctx,
                component_id.get_value(),
                &err,
                refetch_and_reset.clone(),
            );
        }
        None => { /* saving state - do nothing */ }
    });

    // --- Event Handlers ---
    // save function for save button
    let is_dispatching = RwSignal::new(false);
    let on_save = move || {
        is_dispatching.set(true);
        // get diffs
        let tournament_diff = tournament_editor_context.get_tournament_diff();
        let stages_diff = tournament_editor_context.get_stages_diff();
        let groups_diff = tournament_editor_context.get_groups_diff();
        // clear state
        tournament_editor_context.clear();
        // dispatch saves
        if let Some(tournament) = tournament_diff {
            save_tournament_base.dispatch(SaveTournamentBase { tournament });
        }
        for changed_stage in stages_diff {
            save_stage.dispatch(SaveStage {
                stage: changed_stage,
            });
        }
        for _changed_group in groups_diff {
            // ToDo: implement SaveGroup server action and dispatch here
        }
        is_dispatching.set(false);
    };

    // Sync pending state to global context
    Effect::new(move || {
        let busy = save_tournament_base.pending().get()
            || save_stage.pending().get()
            || is_dispatching.get();

        tournament_editor_context.set_busy(busy);
    });

    // --- Validation Logic ---
    let current_tournament_base = Memo::new(move |_| {
        if let Some(s_id) = sport_id()
            && let Some(mut tb) = tournament_editor_context.get_tournament_untracked()
        {
            // Construct the mode explicitly combining the variant selection and the specific input signal
            let mode = match set_mode.get() {
                TournamentMode::SwissSystem { .. } => TournamentMode::SwissSystem {
                    num_rounds: set_num_rounds_swiss.get(),
                },
                other => other,
            };

            // unwrap_or_default is safe here, because component will be unmounted, if sport_id is None.
            tb.set_id_version(set_id_version.get())
                .set_name(set_name.get())
                .set_sport_id(s_id)
                .set_num_entrants(set_entrants.get())
                .set_tournament_type(t_type.get())
                .set_tournament_mode(mode)
                .set_tournament_state(state.get());
            Some(tb)
        } else {
            None
        }
    });

    // Sync to Global State
    Effect::new(move || {
        if let Some(current_tournament) = current_tournament_base.get() {
            tournament_editor_context.set_tournament(current_tournament.clone(), false);
        }
    });

    // Validation runs against the constantly updated Memo
    let validation_result = move || {
        if let Some(current) = current_tournament_base.get() {
            current.validate()
        } else {
            Ok(())
        }
    };

    // error messages for form fields
    let name_error = Signal::derive(move || is_field_valid(validation_result).run("name"));
    let entrants_error =
        Signal::derive(move || is_field_valid(validation_result).run("num_entrants"));
    // Only show Swiss Round logic errors if Swiss Mode is active
    let rounds_error = Signal::derive(move || {
        if let TournamentMode::SwissSystem { .. } = set_mode.get() {
            is_field_valid(validation_result).run("mode.num_rounds")
        } else {
            None
        }
    });

    view! {
        <div
            class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6"
            data-testid="tournament-editor-root"
        >
            <div class="w-full flex justify-between items-center pb-4">
                <h2 class="text-3xl font-bold" data-testid="tournament-editor-title">
                    {move || {
                        if is_new.get() { "Plan New Tournament" } else { "Edit Tournament" }
                    }}
                </h2>
            </div>
            <Show
                when=move || is_new.get() || tournament_id().is_some()
                fallback=|| {
                    view! {
                        <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                            <span class="icon-[heroicons--clipboard-document-list] w-24 h-24 mb-4"></span>
                            <p class="text-2xl font-bold text-center">
                                "Please select a tournament from the list."
                            </p>
                        </div>
                    }
                }
            >
                // Card wrapping Form and Stage Links
                <div class="card w-full bg-base-100 shadow-xl">
                    <div class="card-body">
                        // --- Form Area ---
                        <Transition fallback=move || {
                            view! {
                                <div class="w-full flex justify-center py-8">
                                    <span class="loading loading-spinner loading-lg"></span>
                                </div>
                            }
                        }>
                            {move || {
                                let _ = tournament_res.get();
                                // Access the resource to trigger loading state

                                view! {
                                    <fieldset
                                        disabled=move || {
                                            tournament_editor_context.is_busy()
                                                || page_err_ctx.has_errors()
                                                || matches!(
                                                    state.get(),
                                                    TournamentState::ActiveStage(_) | TournamentState::Finished
                                                )
                                        }
                                        class="contents"
                                        data-testid="tournament-editor-form"
                                    >
                                        <div class="w-full grid grid-cols-1 md:grid-cols-2 gap-6">

                                            <ValidatedTextInput
                                                label="Tournament Name"
                                                name="tournament-name"
                                                value=set_name
                                                error_message=name_error
                                                is_new=is_new
                                                on_blur=move || {
                                                    if let Some(current_tournament) = current_tournament_base
                                                        .get()
                                                    {
                                                        set_name.set(current_tournament.get_name().to_string());
                                                    }
                                                }
                                            />

                                            <ValidatedNumberInput
                                                label="Number of Entrants"
                                                name="tournament-entrants"
                                                value=set_entrants
                                                error_message=entrants_error
                                                is_new=is_new
                                                min="2".to_string()
                                            />

                                            <EnumSelect
                                                label="Mode"
                                                name="tournament-mode"
                                                value=set_mode
                                            />

                                            <Show when=move || {
                                                matches!(set_mode.get(), TournamentMode::SwissSystem { .. })
                                            }>
                                                <ValidatedNumberInput
                                                    label="Rounds (Swiss System)"
                                                    name="tournament-swiss-num_rounds"
                                                    value=set_num_rounds_swiss
                                                    error_message=rounds_error
                                                    is_new=is_new
                                                    min="1".to_string()
                                                />
                                            </Show>

                                        </div>
                                    </fieldset>
                                    // stages editor links
                                    {move || match set_mode.get() {
                                        TournamentMode::SingleStage => {
                                            // set up single stage editor
                                            // with single stage we only have one group in stage
                                            // therefore we "skip" the stage editor and go directly to group editor
                                            view! {
                                                <div class="w-full mt-6">
                                                    <A
                                                        href=move || url_route_with_sub_path("0/0")
                                                        attr:class="btn btn-secondary w-full h-auto min-h-[4rem] text-lg shadow-md"
                                                        attr:data-testid="link-configure-single-stage"
                                                        scroll=false
                                                    >
                                                        <span class="icon-[heroicons--user-group] w-6 h-6 mr-2"></span>
                                                        "Edit Single Stage"
                                                    </A>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        TournamentMode::PoolAndFinalStage => {
                                            // set up pool and final stage editor
                                            // with pool and final stage we have two stages to configure
                                            view! {
                                                <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full mt-6">
                                                    <A
                                                        href=move || url_route_with_sub_path("0")
                                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                                        attr:data-testid="link-configure-pool-stage"
                                                        scroll=false
                                                    >
                                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                                        "Edit Pool Stage"
                                                    </A>
                                                    <A
                                                        href=move || url_route_with_sub_path("1")
                                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                                        attr:data-testid="link-configure-final-stage"
                                                        scroll=false
                                                    >
                                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                                        "Edit Final Stage"
                                                    </A>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        TournamentMode::TwoPoolStagesAndFinalStage => {
                                            // set up two pool stages and final stage editor
                                            // with pool and final stage we have three stages to configure
                                            view! {
                                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full mt-6">
                                                    <A
                                                        href=move || url_route_with_sub_path("0")
                                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                                        attr:data-testid="link-configure-first-pool-stage"
                                                        scroll=false
                                                    >
                                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                                        "Edit First Pool Stage"
                                                    </A>
                                                    <A
                                                        href=move || url_route_with_sub_path("1")
                                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                                        attr:data-testid="link-configure-second-pool-stage"
                                                        scroll=false
                                                    >
                                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                                        "Edit Second Pool Stage"
                                                    </A>
                                                    <A
                                                        href=move || url_route_with_sub_path("2")
                                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                                        attr:data-testid="link-configure-final-stage"
                                                        scroll=false
                                                    >
                                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                                        "Edit Final Stage"
                                                    </A>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        TournamentMode::SwissSystem { .. } => {
                                            // set up swiss system stage editor
                                            // with swiss system we only have one group in stage
                                            // therefore we "skip" the stage editor and go directly to group editor
                                            view! {
                                                <div class="w-full mt-6">
                                                    <A
                                                        href=move || url_route_with_sub_path("0/0")
                                                        attr:class="btn btn-secondary w-full h-auto min-h-[4rem] text-lg shadow-md"
                                                        attr:data-testid="link-configure-swiss-system"
                                                        scroll=false
                                                    >
                                                        <span class="icon-[heroicons--user-group] w-6 h-6 mr-2"></span>
                                                        "Edit Swiss System"
                                                    </A>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }}
                                }
                            }}
                        </Transition>
                    </div>
                </div>
            </Show>

            <Outlet />

            {move || {
                let on_cancel = on_cancel.clone();
                if !is_new.get() && tournament_id().is_none() {
                    view! {}.into_any()
                } else {
                    view! {
                        // --- Action Buttons ---
                        <div class="w-full flex justify-end gap-4 pt-6">
                            <button
                                class="btn btn-ghost"
                                data-testid="btn-tournament-cancel"
                                on:click=move |_| on_cancel()
                                disabled=move || tournament_editor_context.is_busy()
                            >
                                "Cancel"
                            </button>

                            <button
                                class="btn btn-primary"
                                data-testid="btn-tournament-save"
                                on:click=move |_| on_save()
                                disabled=move || {
                                    tournament_editor_context.is_busy() || page_err_ctx.has_errors()
                                        || !tournament_editor_context.is_valid() || !is_changed()
                                }
                            >
                                {move || {
                                    if tournament_editor_context.is_busy() {
                                        view! {
                                            <span class="loading loading-spinner"></span>
                                            "Saving..."
                                        }
                                            .into_any()
                                    } else {
                                        "Save Tournament".into_any()
                                    }
                                }}
                            </button>
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}
