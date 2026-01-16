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
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{SportParams, TournamentBaseParams},
    server_fn::tournament_base::{SaveTournamentBase, load_tournament_base},
    state::{
        error_state::PageErrorContext,
        toast_state::{ToastContext, ToastVariant},
        tournament_editor_state::TournamentEditorState,
    },
};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    components::A,
    hooks::{use_navigate, use_query},
    nested_router::Outlet,
};
use uuid::Uuid;

#[component]
pub fn EditTournament() -> impl IntoView {
    // --- Initialize context for creating and editing tournaments ---
    let tournament_editor_state = RwSignal::new(TournamentEditorState::new());
    provide_context::<RwSignal<TournamentEditorState>>(tournament_editor_state);

    let is_changed = move || tournament_editor_state.read().is_changed();

    // --- Hooks, Navigation & toast/ error state ---
    let UseQueryNavigationReturn {
        nav_url,
        update,
        relative_sub_url,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    let sport_id_query = use_query::<SportParams>();
    let tournament_id_query = use_query::<TournamentBaseParams>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let toast_ctx = expect_context::<ToastContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    // None = effectively navigating away or invalid state.
    let sport_id = move || sport_id_query.get().ok().and_then(|p| p.sport_id);

    let tournament_id = move || tournament_id_query.get().ok().and_then(|p| p.tournament_id);

    let is_new = Signal::derive(move || tournament_id().is_none());

    // Form Signals
    let set_id_version = RwSignal::new(IdVersion::New);
    let set_name = RwSignal::new("".to_string());
    let set_entrants = RwSignal::new(0_u32);
    let t_type = StoredValue::new(TournamentType::Scheduled);
    let set_mode = RwSignal::new(TournamentMode::SingleStage);
    let set_num_rounds_swiss = RwSignal::new(0_u32);
    let state = StoredValue::new(TournamentState::Draft);

    let is_disabled_stage = move |num: u32| match state.get_value() {
        TournamentState::ActiveStage(active_stage) => active_stage >= num,
        TournamentState::Finished => true,
        _ => false,
    };

    // --- Server Resources & Actions  ---
    // load tournament base resource
    let tournament_res = Resource::new(
        move || (tournament_id(), sport_id()),
        move |(maybe_t_id, maybe_s_id)| async move {
            if let Some(s_id) = maybe_s_id {
                if let Some(t_id) = maybe_t_id {
                    match load_tournament_base(t_id).await {
                        Ok(Some(tournament)) => Ok(Some(tournament)),
                        Ok(None) => Err(AppError::ResourceNotFound(
                            "Tournament Base".to_string(),
                            t_id,
                        )),
                        Err(e) => Err(e),
                    }
                } else {
                    let mut tournament = TournamentBase::default();
                    tournament
                        .set_sport_id(s_id)
                        .set_id_version(IdVersion::NewWithId(Uuid::new_v4()));

                    Ok(Some(tournament))
                }
            } else {
                Ok(None)
            }
        },
    );

    // handle successful load
    Effect::new(move || {
        if let Some(Ok(Some(tournament))) = tournament_res.get() {
            set_id_version.set(tournament.get_id_version());
            set_name.set(tournament.get_name().to_string());
            set_entrants.set(tournament.get_num_entrants());
            t_type.set_value(tournament.get_tournament_type());
            set_mode.set(tournament.get_tournament_mode());
            if let TournamentMode::SwissSystem { num_rounds } = tournament.get_tournament_mode() {
                set_num_rounds_swiss.set(num_rounds);
            }
            state.set_value(tournament.get_tournament_state());

            tournament_editor_state.update(|state| {
                state.set_tournament(tournament.clone(), !is_new.get());
            });
        }
    });

    // save tournament base action
    let save_tournament_base = ServerAction::<SaveTournamentBase>::new();

    // derived signals of action
    let is_pending = save_tournament_base.pending();

    // handle successful save
    Effect::new({
        let navigate = navigate.clone();
        move || {
            if let Some(Ok(tb)) = save_tournament_base.value().get() {
                save_tournament_base.clear();
                toast_ctx.add("Tournament saved successfully", ToastVariant::Success);
                update(
                    "tournament_id",
                    &tb.get_id().map(|id| id.to_string()).unwrap_or_default(),
                );
                navigate(
                    &nav_url.get(),
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
        }
    });

    // --- Event Handlers ---
    // save function for save button
    let on_save = move || {
        if let Some(tournament) = tournament_editor_state.read().get_tournament_diff() {
            save_tournament_base.dispatch(SaveTournamentBase { tournament });
        }
        for _changed_stage in tournament_editor_state.read().get_stages_diff() {
            // ToDo: save stages
        }
    };

    // retry function for error handling
    let refetch_and_reset = move || {
        save_tournament_base.clear();
        tournament_res.refetch();
    };

    // cancel function for cancel button and error handling
    let on_cancel = {
        let navigate = navigate.clone();
        move || {
            // unwrap_or_default() is safe here, because component will be unmounted,
            // if sport_id is None
            let s_id = sport_id().unwrap_or_default();
            let _ = navigate(&format!("/?sport_id={}", s_id), Default::default());
        }
    };

    // Handle read and write errors
    Effect::new({
        let on_cancel = on_cancel.clone();
        move || {
            if let Some(Err(err)) = tournament_res.get() {
                handle_read_error(
                    &page_err_ctx,
                    component_id.get_value(),
                    &err,
                    refetch_and_reset.clone(),
                    on_cancel.clone(),
                );
            }
        }
    });
    Effect::new(move || {
        if let Some(Err(err)) = save_tournament_base.value().get() {
            handle_write_error(
                &page_err_ctx,
                &toast_ctx,
                component_id.get_value(),
                &err,
                refetch_and_reset.clone(),
            );
        }
    });

    // --- Validation Logic ---
    let current_tournament_base = Memo::new(move |_| {
        let mut tb = TournamentBase::default();

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
            .set_sport_id(sport_id().unwrap_or_default())
            .set_num_entrants(set_entrants.get())
            .set_tournament_type(t_type.get_value())
            .set_tournament_mode(mode)
            .set_tournament_state(state.get_value());
        tb
    });

    // Validation runs against the constantly updated Memo
    let validation_result = move || current_tournament_base.get().validate();
    let is_valid_tournament = move || validation_result().is_ok();

    // Sync to Global State: Only if valid!
    Effect::new(move || {
        if is_valid_tournament() {
            tournament_editor_state.update(|state| {
                state.set_tournament(current_tournament_base.get(), false);
            });
        }
    });

    // Helper for error messages in the inputs
    // derive creates a read-only signal for the inputs
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
                <h2
                    class="text-3xl font-bold"
                    data-testid="tournament-editor-title"
                >
                    {move || {
                        if is_new.get() { "Plan New Tournament" } else { "Edit Tournament" }
                    }}
                </h2>
            </div>

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
                        <fieldset
                            disabled=move || {
                                is_pending.try_get().unwrap_or(false) || page_err_ctx.has_errors()
                                    || matches!(
                                        state.get_value(),
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
                                />

                                <ValidatedNumberInput
                                    label="Number of Entrants"
                                    name="tournament-entrants"
                                    value=set_entrants
                                    error_message=entrants_error
                                    is_new=is_new
                                    min="2".to_string()
                                />

                                <EnumSelect label="Mode" name="tournament-mode" value=set_mode />

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
                    </Transition>

                    // stages editor links
                    {move || match set_mode.get() {
                        TournamentMode::SingleStage => {
                            // set up single stage editor
                            // with single stage we only have one group in stage
                            // therefore we "skip" the stage editor and go directly to group editor
                            view! {
                                <div class="w-full mt-6">
                                    <A
                                        href=relative_sub_url("/0/0")
                                        attr:class="btn btn-secondary w-full h-auto min-h-[4rem] text-lg shadow-md"
                                        attr:data-testid="link-configure-single-stage"
                                        attr:disabled=is_disabled_stage(0)
                                    >
                                        <span class="icon-[heroicons--user-group] w-6 h-6 mr-2"></span>
                                        "Configure Single Stage"
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
                                        href=relative_sub_url("/0")
                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                        attr:data-testid="link-configure-pool-stage"
                                        attr:disabled=is_disabled_stage(0)
                                    >
                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                        "Configure Pool Stage"
                                    </A>
                                    <A
                                        href=relative_sub_url("/1")
                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                        attr:data-testid="link-configure-final-stage"
                                        attr:disabled=is_disabled_stage(1)
                                    >
                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                        "Configure Final Stage"
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
                                        href=relative_sub_url("/0")
                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                        attr:data-testid="link-configure-first-pool-stage"
                                        attr:disabled=is_disabled_stage(0)
                                    >
                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                        "Configure First Pool Stage"
                                    </A>
                                    <A
                                        href=relative_sub_url("/1")
                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                        attr:data-testid="link-configure-second-pool-stage"
                                        attr:disabled=is_disabled_stage(1)
                                    >
                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                        "Configure Second Pool Stage"
                                    </A>
                                    <A
                                        href=relative_sub_url("/2")
                                        attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                        attr:data-testid="link-configure-final-stage"
                                        attr:disabled=is_disabled_stage(2)
                                    >
                                        <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                        "Configure Final Stage"
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
                                        href=relative_sub_url("/0/0")
                                        attr:class="btn btn-secondary w-full h-auto min-h-[4rem] text-lg shadow-md"
                                        attr:data-testid="link-configure-swiss-system"
                                        attr:disabled=is_disabled_stage(0)
                                    >
                                        <span class="icon-[heroicons--user-group] w-6 h-6 mr-2"></span>
                                        "Configure Swiss System"
                                    </A>
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>

            <Outlet />

            // --- Action Buttons ---
            <div class="w-full flex justify-end gap-4 pt-6">
                <button
                    class="btn btn-ghost"
                    data-testid="btn-tournament-cancel"
                    on:click=move |_| on_cancel()
                    disabled=move || is_pending.try_get().unwrap_or(false)
                >
                    "Cancel"
                </button>

                <button
                    class="btn btn-primary"
                    data-testid="btn-tournament-save"
                    on:click=move |_| on_save()
                    disabled=move || {
                        is_pending.try_get().unwrap_or(false) || page_err_ctx.has_errors()
                            || !is_valid_tournament() || !is_changed()
                    }
                >
                    {move || {
                        if is_pending.try_get().unwrap_or(false) {
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
        </div>
    }
}
