//! Edit tournament stage component

use app_core::Stage;
use app_utils::{
    components::inputs::NumberInputWithValidation,
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
    params::{use_stage_number_params, use_tournament_base_id_query},
    server_fn::stage::load_stage_by_number,
    state::{
        error_state::PageErrorContext,
        tournament_editor::{TournamentEditorContext, TournamentRefetchContext},
    },
};
use leptos::prelude::*;
use leptos_router::{components::A, nested_router::Outlet};
use uuid::Uuid;

#[component]
pub fn LoadTournamentStage() -> impl IntoView {
    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    let refetch_trigger = expect_context::<TournamentRefetchContext>();

    // --- url parameters & queries ---
    let tournament_id = use_tournament_base_id_query();
    let active_stage_number = use_stage_number_params();

    // --- Resource to load tournament stage ---
    let stage_res = Resource::new(
        move || {
            (
                tournament_id.get(),
                active_stage_number.get(),
                refetch_trigger.track_fetch_trigger.get(),
            )
        },
        move |(maybe_t_id, maybe_s_num, _track_refetch)| async move {
            if let Some(t_id) = maybe_t_id
                && let Some(stage_number) = maybe_s_num
            {
                load_stage_by_number(t_id, stage_number).await
            } else {
                Ok(None)
            }
        },
    );

    // retry function for error handling
    let refetch = Callback::new(move |()| {
        refetch_trigger.trigger_refetch();
        stage_res.refetch();
    });

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    view! {
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
                {move || {
                    stage_res
                        .and_then(|may_be_s| {
                            view! { <EditTournamentStage stage=*may_be_s /> }
                        })
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
pub fn EditTournamentStage(stage: Option<Stage>) -> impl IntoView {
    // --- Get context for creating and editing tournaments ---
    let active_stage_number = use_stage_number_params();

    let tournament_editor_context = expect_context::<TournamentEditorContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();

    Effect::new(move || {
        if let Some(s) = stage {
            tournament_editor_context.set_stage(s);
        } else if let Some(stage_number) = active_stage_number.get() {
            tournament_editor_context.new_stage(stage_number);
        }
    });

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_matched_route, ..
    } = use_query_navigation();

    let editor_title = move || {
        if let Some(sn) = tournament_editor_context.active_stage_number.get()
            && let Some(title) = tournament_editor_context
                .base_mode
                .get()
                .and_then(|m| m.get_stage_name(sn))
        {
            format!("Edit {}", title)
        } else {
            "Edit Tournament Stage".to_string()
        }
    };

    view! {
        // hide stage editor for single stage and swiss system tournaments
        <Show when=move || !tournament_editor_context.is_hiding_stage_editor.get()>
            // Card wrapping Form and Group Links
            <div class="card w-full bg-base-100 shadow-xl">
                <div class="card-body">
                    // --- Form Area ---
                    <div
                        class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6"
                        data-testid="stage-editor-root"
                    >
                        <div class="w-full flex justify-between items-center pb-4">
                            <h2 class="text-3xl font-bold" data-testid="stage-editor-title">
                                {move || editor_title()}
                            </h2>
                        </div>
                        // we have to use try_get here to avoid runtime panics, because
                        // page_err_ctx "lives" independent of tournament_editor_context
                        <fieldset
                            disabled=move || {
                                page_err_ctx.has_errors()
                                    || tournament_editor_context
                                        .is_disabled_stage_editing
                                        .try_get()
                                        .unwrap_or(false)
                                    || tournament_editor_context.is_busy.try_get().unwrap_or(false)
                                    || !tournament_editor_context
                                        .is_stage_initialized
                                        .try_get()
                                        .unwrap_or(false)
                            }
                            class="contents"
                            data-testid="stage-editor-form"
                        >
                            <div class="w-full max-w-md grid grid-cols-1 gap-6">
                                <NumberInputWithValidation
                                    label="Number of Groups"
                                    name="stage-num-groups"
                                    value=tournament_editor_context.stage_num_groups
                                    set_value=tournament_editor_context.set_stage_num_groups
                                    validation_result=tournament_editor_context.validation_result
                                    min="1".to_string()
                                    object_id=tournament_editor_context.active_stage_id
                                    field="num_groups"
                                />
                            </div>
                        // group editor links
                        </fieldset>
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full mt-6">
                            <For
                                each=move || {
                                    0..tournament_editor_context
                                        .stage_num_groups
                                        .get()
                                        .unwrap_or_default()
                                }
                                key=|i| *i
                                children=move |i| {
                                    view! {
                                        <A
                                            href=move || url_matched_route(
                                                MatchedRouteHandler::Extend(&i.to_string()),
                                            )
                                            attr:class="btn btn-secondary h-auto min-h-[4rem] text-lg shadow-md"
                                            attr:data-testid=format!("link-configure-group-{}", i)
                                            scroll=false
                                        >
                                            <span class="icon-[heroicons--rectangle-stack] w-6 h-6 mr-2"></span>
                                            {format!("Edit Group {}", i + 1)}
                                        </A>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>
            </div>
        </Show>
        <Outlet />
    }
}
