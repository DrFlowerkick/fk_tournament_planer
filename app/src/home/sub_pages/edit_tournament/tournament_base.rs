//! create or edit a tournament

use app_core::{TournamentEditorState, TournamentMode, TournamentState};
use app_utils::{
    components::inputs::{
        EnumSelectWithValidation, NumberInputWithValidation, TextInputWithValidation,
    },
    error::{
        AppError,
        strategy::{handle_general_error, handle_read_error, handle_write_error},
    },
    hooks::{
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
        tournament_editor::TournamentEditorContext,
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
            tournament_editor_context.track_url_validation.track();

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

    // --- Setter Callbacks for Tournament Base Properties ---
    let set_name = Callback::new(move |name| tournament_editor_context.set_base_name.set(name));
    let set_num_entrants =
        Callback::new(move |num| tournament_editor_context.set_base_num_entrants.set(num));
    let set_mode = Callback::new(move |mode| {
        tournament_editor_context.set_base_mode.set(mode);
    });
    let set_num_rounds_swiss = Callback::new(move |num| {
        tournament_editor_context
            .set_base_num_rounds_swiss_system
            .set(num)
    });

    // --- Server Resources & Actions  ---
    // load tournament base resource
    let tournament_res = Resource::new(
        move || (tournament_id(), sport_id()),
        move |(maybe_t_id, maybe_s_id)| async move {
            // ToDo: do we really need to check sport_id here?
            if let Some(_s_id) = maybe_s_id
                && let Some(t_id) = maybe_t_id
            {
                return match load_tournament_base(t_id).await {
                    Ok(None) => Err(AppError::ResourceNotFound(
                        "Tournament Base".to_string(),
                        t_id,
                    )),
                    load_result => load_result,
                };
            }
            Ok(None)
        },
    );

    // save tournament base action
    let save_tournament_base = ServerAction::<SaveTournamentBase>::new();

    // save stage action
    let save_stage = ServerAction::<SaveStage>::new();

    // retry function for error handling
    let refetch_and_reset = Callback::new(move |()| {
        save_tournament_base.clear();
        save_stage.clear();
        tournament_res.refetch();
    });

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // handle save tournament base results
    Effect::new({
        let navigate = navigate.clone();
        move || match save_tournament_base.value().get() {
            Some(Ok(tb)) => {
                save_tournament_base.clear();
                toast_ctx.add("Tournament saved successfully", ToastVariant::Success);
                let nav_url =
                    url_with_update_query("tournament_id", &tb.get_id().to_string(), None);
                // set saved tb as origin in editor context before navigation to prevent
                // load effect to reset the editor state
                // ToDo: is this still required? Uncomment if yes
                //tournament_editor_context.set_base(tb);

                if tournament_id().is_some() {
                    // if it was a new tournament, trigger refetch to load the full data
                    tournament_res.refetch();
                } else {
                    // else navigate directly
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

    // handle save stage results
    Effect::new(move || match save_stage.value().get() {
        Some(Ok(_stage)) => {
            save_stage.clear();
            toast_ctx.add("Stage saved successfully", ToastVariant::Success);
            // trigger refetch of stage resource
            tournament_editor_context.trigger_stage_refetch();
        }
        Some(Err(err)) => {
            handle_write_error(
                &page_err_ctx,
                &toast_ctx,
                component_id.get_value(),
                &err,
                refetch_and_reset,
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
        let base_diff = tournament_editor_context.collect_base_diff();
        let stages_diff = tournament_editor_context.collect_stages_diff();
        let groups_diff = tournament_editor_context.collect_groups_diff();
        // dispatch saves
        if let Some(base) = base_diff {
            // ToDo: rename later tournament to base in SaveTournamentBase
            save_tournament_base.dispatch(SaveTournamentBase { tournament: base });
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
                            <ErrorBoundary fallback=move |errors| {
                                for (_err_id, err) in errors.get().into_iter() {
                                    let e = err.into_inner();
                                    if let Some(app_err) = e.downcast_ref::<AppError>() {
                                        handle_read_error(
                                            &page_err_ctx,
                                            component_id.get_value(),
                                            app_err,
                                            refetch_and_reset,
                                            on_cancel,
                                        );
                                    } else {
                                        handle_general_error(
                                            &page_err_ctx,
                                            component_id.get_value(),
                                            "An unexpected error occurred.",
                                            None,
                                            on_cancel,
                                        );
                                    }
                                }
                            }>
                                // check if we have new or existing tournament
                                // In case of new tournament, initialize editor with new base if
                                // -> no base is present yet
                                // -> an already saved base is in context (meaning a click on New Tournament
                                // while editing an existing one)
                                {move || {
                                    tournament_res
                                        .and_then(|may_be_t| {
                                            match may_be_t {
                                                Some(tournament) => {
                                                    tournament_editor_context.set_base(tournament.clone());
                                                }
                                                None => {
                                                    if let Some(s_id) = sport_id()
                                                        && matches!(
                                                            tournament_editor_context.state.get_untracked(),
                                                            TournamentEditorState::None | TournamentEditorState::Edit
                                                        )
                                                    {
                                                        tournament_editor_context.new_base(s_id);
                                                    }
                                                }
                                            }
                                        })
                                }}
                                <fieldset
                                    disabled=move || {
                                        tournament_editor_context.is_busy.get()
                                            || page_err_ctx.has_errors()
                                            || matches!(
                                                tournament_editor_context.base_state.get(),
                                                Some(TournamentState::ActiveStage(_))
                                                | Some(TournamentState::Finished)
                                            )
                                    }
                                    class="contents"
                                    data-testid="tournament-editor-form"
                                >
                                    <div class="w-full grid grid-cols-1 md:grid-cols-2 gap-6">

                                        <TextInputWithValidation
                                            label="Tournament Name"
                                            name="tournament-name"
                                            value=tournament_editor_context.base_name
                                            set_value=set_name
                                            validation_result=tournament_editor_context
                                                .validation_result
                                            object_id=tournament_editor_context.base_id
                                            field="name"
                                        />

                                        <NumberInputWithValidation
                                            label="Number of Entrants"
                                            name="tournament-entrants"
                                            value=tournament_editor_context.base_num_entrants
                                            set_value=set_num_entrants
                                            validation_result=tournament_editor_context
                                                .validation_result
                                            object_id=tournament_editor_context.base_id
                                            field="num_entrants"
                                            min="2".to_string()
                                        />

                                        <EnumSelectWithValidation
                                            label="Mode"
                                            name="tournament-mode"
                                            value=tournament_editor_context.base_mode
                                            set_value=set_mode
                                            validation_result=tournament_editor_context
                                                .validation_result
                                            object_id=None
                                            field="No Direct Validation"
                                        />

                                        <Show when=move || {
                                            matches!(
                                                tournament_editor_context.base_mode.get(),
                                                Some(TournamentMode::SwissSystem { .. })
                                            )
                                        }>
                                            <NumberInputWithValidation
                                                label="Rounds (Swiss System)"
                                                name="tournament-swiss-num_rounds"
                                                value=tournament_editor_context.base_num_rounds_swiss_system
                                                set_value=set_num_rounds_swiss
                                                validation_result=tournament_editor_context
                                                    .validation_result
                                                object_id=tournament_editor_context.base_id
                                                field="mod.num_rounds"
                                                min="1".to_string()
                                            />
                                        </Show>

                                    </div>
                                // stages editor links
                                </fieldset>
                                {move || match tournament_editor_context.base_mode.get() {
                                    Some(TournamentMode::SingleStage) => {
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
                                    Some(TournamentMode::PoolAndFinalStage) => {
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
                                    Some(TournamentMode::TwoPoolStagesAndFinalStage) => {
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
                                    Some(TournamentMode::SwissSystem { .. }) => {
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
                                    None => ().into_any(),
                                }}

                            </ErrorBoundary>
                        </Transition>
                    </div>
                </div>
            </Show>

            <Outlet />

            {move || {
                if !is_new.get() && tournament_id().is_none() {
                    view! {}.into_any()
                } else {
                    view! {
                        // --- Action Buttons ---
                        <div class="w-full flex justify-end gap-4 pt-6">
                            <button
                                class="btn btn-ghost"
                                data-testid="btn-tournament-cancel"
                                on:click=move |_| on_cancel.run(())
                                disabled=move || tournament_editor_context.is_busy.get()
                            >
                                "Cancel"
                            </button>

                            <button
                                class="btn btn-primary"
                                data-testid="btn-tournament-save"
                                on:click=move |_| on_save()
                                disabled=move || {
                                    tournament_editor_context.is_busy.get()
                                        || page_err_ctx.has_errors()
                                        || tournament_editor_context
                                            .validation_result
                                            .get()
                                            .is_err() || !tournament_editor_context.is_changed.get()
                                }
                            >
                                {move || {
                                    if tournament_editor_context.is_busy.get() {
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
