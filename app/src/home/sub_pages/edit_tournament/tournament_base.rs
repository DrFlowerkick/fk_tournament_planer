//! create or edit a tournament

use app_core::{TournamentBase, TournamentEditor, TournamentMode};
use app_utils::{
    components::inputs::{
        EnumSelectWithValidation, NumberInputWithValidation, TextInputWithValidation,
    },
    error::{
        AppError,
        strategy::{handle_general_error, handle_read_error},
    },
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{
            MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
        },
    },
    params::{use_sport_id_query, use_tournament_base_id_query},
    server_fn::tournament_base::load_tournament_base,
    state::{
        error_state::PageErrorContext,
        tournament_editor::{TournamentEditorContext, TournamentRefetchContext},
    },
};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_url, nested_router::Outlet};
use uuid::Uuid;

#[component]
pub fn LoadTournament() -> impl IntoView {
    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });
    let refetch_trigger = TournamentRefetchContext::new();
    provide_context(refetch_trigger);

    // --- url queries ---
    let sport_id = use_sport_id_query();
    let tournament_id = use_tournament_base_id_query();

    // --- Resource to load tournament base ---
    let base_res = Resource::new(
        move || {
            (
                tournament_id.get(),
                sport_id.get(),
                refetch_trigger.track_fetch_trigger.get(),
            )
        },
        move |(maybe_t_id, maybe_s_id, _track_refetch)| async move {
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
        },
    );

    // retry function for error handling
    let refetch = Callback::new(move |()| {
        refetch_trigger.trigger_refetch();
    });

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    view! {
        <Transition fallback=move || {
            view! {
                <div class="w-full flex flex-col items-center justify-center py-12 opacity-50">
                    <span class="loading loading-spinner w-24 h-24 mb-4"></span>
                    <p class="text-2xl font-bold text-center">"Loading tournament data..."</p>
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
                            refetch,
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
                    base_res
                        .and_then(|may_be_t| {
                            view! { <EditTournament base=may_be_t.clone() /> }
                        })
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn EditTournament(base: Option<TournamentBase>) -> impl IntoView {
    // --- prepare initial tournament editor state ---
    let sport_id = use_sport_id_query();
    let url = use_url();

    let mut tournament_editor = TournamentEditor::new();
    let (show_form, is_new) = if let Some(b) = base {
        tournament_editor.set_base(b);
        (true, false)
    } else if let Some(s_id) = sport_id.get_untracked() {
        tournament_editor.new_base(s_id);
        let is_new = url.get_untracked().path().starts_with("/new-tournament");
        (is_new, is_new)
    } else {
        (false, false)
    };

    // --- Initialize context for creating and editing tournaments ---
    let tournament_editor_context = TournamentEditorContext::new(tournament_editor);
    provide_context(tournament_editor_context);
    let page_err_ctx = expect_context::<PageErrorContext>();

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_matched_route, ..
    } = use_query_navigation();

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    view! {
        <div
            class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6"
            data-testid="tournament-editor-root"
        >
            <div class="w-full flex justify-between items-center pb-4">
                <h2 class="text-3xl font-bold" data-testid="tournament-editor-title">
                    {move || { if is_new { "Plan New Tournament" } else { "Edit Tournament" } }}
                </h2>
            </div>
            <Show
                when=move || show_form
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
                        // we have to use try_get here to avoid runtime panics, because
                        // page_err_ctx "lives" independent of tournament_editor_context
                        <fieldset
                            disabled=move || {
                                page_err_ctx.has_errors()
                                    || tournament_editor_context
                                        .is_disabled_base_editing
                                        .try_get()
                                        .unwrap_or(false)
                                    || tournament_editor_context.is_busy.try_get().unwrap_or(false)
                                    || !tournament_editor_context
                                        .is_base_initialized
                                        .try_get()
                                        .unwrap_or(false)
                            }
                            class="contents"
                            data-testid="tournament-editor-form"
                        >
                            <div class="w-full grid grid-cols-1 md:grid-cols-2 gap-6">

                                <TextInputWithValidation
                                    label="Tournament Name"
                                    name="tournament-name"
                                    value=tournament_editor_context.base_name
                                    set_value=tournament_editor_context.set_base_name
                                    validation_result=tournament_editor_context.validation_result
                                    object_id=tournament_editor_context.base_id
                                    field="name"
                                />

                                <NumberInputWithValidation
                                    label="Number of Entrants"
                                    name="tournament-entrants"
                                    value=tournament_editor_context.base_num_entrants
                                    set_value=tournament_editor_context.set_base_num_entrants
                                    validation_result=tournament_editor_context.validation_result
                                    object_id=tournament_editor_context.base_id
                                    field="num_entrants"
                                    min="2".to_string()
                                />

                                <EnumSelectWithValidation
                                    label="Mode"
                                    name="tournament-mode"
                                    value=tournament_editor_context.base_mode
                                    set_value=tournament_editor_context.set_base_mode
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
                                        set_value=tournament_editor_context
                                            .set_base_num_rounds_swiss_system
                                        validation_result=tournament_editor_context
                                            .validation_result
                                        object_id=tournament_editor_context.base_id
                                        field="mode.num_rounds"
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
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("0/0"),
                                            )
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
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("0"),
                                            )
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-pool-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit Pool Stage"
                                        </A>
                                        <A
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("1"),
                                            )
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
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("0"),
                                            )
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-first-pool-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit First Pool Stage"
                                        </A>
                                        <A
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("1"),
                                            )
                                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid="link-configure-second-pool-stage"
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            "Edit Second Pool Stage"
                                        </A>
                                        <A
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("2"),
                                            )
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
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend("0/0"),
                                            )
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
                    </div>
                </div>
            </Show>

            <Outlet />

            <Show when=move || show_form>
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

                    // we have to use try_get here to avoid runtime panics, because
                    // page_err_ctx "lives" independent of tournament_editor_context
                    <button
                        class="btn btn-primary"
                        data-testid="btn-tournament-save"
                        on:click=move |_| tournament_editor_context.save_diff()
                        disabled=move || {
                            !tournament_editor_context.is_changed.try_get().unwrap_or(false)
                                || tournament_editor_context
                                    .validation_result
                                    .try_get()
                                    .map(|res| res.is_err())
                                    .unwrap_or(false)
                                || tournament_editor_context.is_busy.try_get().unwrap_or(false)
                                || page_err_ctx.has_errors()
                        }
                    >
                        <Show
                            when=move || !tournament_editor_context.is_busy.get()
                            fallback=|| {
                                view! { <span class="loading loading-spinner">"Saving..."</span> }
                            }
                        >
                            <span>"Save Tournament"</span>
                        </Show>
                    </button>
                </div>
            </Show>
        </div>
    }
}
