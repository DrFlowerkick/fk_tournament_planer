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
        use_scroll_into_view::use_scroll_h2_into_view,
    },
    params::{ParamQuery, StageNumberParams, TournamentBaseIdQuery},
    server_fn::stage::load_stage_by_number,
    state::{
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        tournament_editor::{TournamentEditorContext, TournamentRefetchContext},
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{components::A, nested_router::Outlet};
use uuid::Uuid;

#[component]
pub fn LoadTournamentStage() -> impl IntoView {
    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    let refetch_trigger = expect_context::<TournamentRefetchContext>();

    // --- url parameters & queries ---
    let tournament_id = TournamentBaseIdQuery::use_param_query();
    let active_stage_number = StageNumberParams::use_param_query();

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
                activity_tracker
                    .track_activity_wrapper(
                        component_id.get_value(),
                        load_stage_by_number(t_id, stage_number),
                    )
                    .await
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
                <div class="card w-full bg-base-100 shadow-xl">
                    <div class="card-body">
                        <div class="w-full flex justify-center py-8">
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    </div>
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
    let active_stage_number = StageNumberParams::use_param_query();

    let tournament_editor_context = expect_context::<TournamentEditorContext>();

    Effect::new(move || {
        if let Some(s) = stage {
            tournament_editor_context.set_stage(s);
        } else if let Some(stage_number) = active_stage_number.get() {
            tournament_editor_context.new_stage(stage_number);
        }
    });

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_matched_route,
        url_is_matched_route,
        ..
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

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        // hide stage editor for single stage and swiss system tournaments
        <Show when=move || !tournament_editor_context.is_hiding_stage_editor.get()>
            // Card wrapping Form and Group Links
            <div class="card w-full bg-base-100 shadow-xl">
                <div class="card-body">
                    // --- Form Area ---
                    <div data-testid="stage-editor-root">
                        <h2 class="card-title" data-testid="stage-editor-title" node_ref=scroll_ref>
                            {move || editor_title()}
                        </h2>
                        <fieldset
                            disabled=move || {
                                tournament_editor_context.is_disabled_stage_editing.get()
                                    || tournament_editor_context.is_busy.get()
                                    || !tournament_editor_context.is_stage_initialized.get()
                            }
                            class="space-y-4 contents"
                            data-testid="stage-editor-form"
                        >
                            <div class="w-full max-w-md grid grid-cols-1 gap-6">
                                <NumberInputWithValidation
                                    label="Number of Groups"
                                    name="stage-num-groups"
                                    data_testid="input-stage-num-groups"
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
        <div class="my-4"></div>
        <Outlet />
    }
}
