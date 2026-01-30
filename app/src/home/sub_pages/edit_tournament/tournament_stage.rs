//! Edit tournament stage component

use app_core::{TournamentMode, TournamentState};
use app_utils::{
    components::inputs::NumberInputWithValidation,
    error::AppError,
    error::strategy::{handle_general_error, handle_read_error},
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{StageParams, TournamentBaseParams},
    server_fn::stage::load_stage_by_number,
    state::{error_state::PageErrorContext, tournament_editor::TournamentEditorContext},
};
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::{use_params, use_query},
    nested_router::Outlet,
};
use uuid::Uuid;

#[component]
pub fn EditTournamentStage() -> impl IntoView {
    // --- Get context for creating and editing tournaments ---
    let tournament_editor_context = expect_context::<TournamentEditorContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        url_route_with_sub_path,
        ..
    } = use_query_navigation();

    let tournament_id_query = use_query::<TournamentBaseParams>();
    let tournament_id = move || {
        if let Ok(t_id_params) = tournament_id_query.get()
            && let Some(t_id) = t_id_params.tournament_id
        {
            Some(t_id)
        } else {
            tournament_editor_context.base_id.get()
        }
    };

    let stage_number_params = use_params::<StageParams>();
    let stage_number = move || {
        if let Ok(snp) = stage_number_params.get()
            && let Some(sn) = snp.stage_number
            && let Some(num_stages) = tournament_editor_context
                .base_mode
                .get()
                .map(|m| m.get_num_of_stages())
            && num_stages > sn
        {
            return Some(sn);
        }
        None
    };
    let editor_title = move || {
        if let Some(sn) = stage_number()
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

    // check invalid stage_number in params
    Effect::new(move || {
        if let Ok(snp) = stage_number_params.get()
            && let Some(sn) = snp.stage_number
            && let Some(num_stages) = tournament_editor_context
                .base_mode
                .get()
                .map(|m| m.get_num_of_stages())
            && num_stages <= sn
        {
            // invalid stage number -> signal url validation
            tournament_editor_context.trigger_url_validation();
        }
    });

    // check if stage is not editable
    let is_active_or_done = move || {
        if let Some(sn) = stage_number()
            && let Some(tournament_state) = tournament_editor_context.base_state.get()
        {
            match tournament_state {
                TournamentState::ActiveStage(acs) => acs >= sn,
                TournamentState::Finished => true,
                _ => false,
            }
        } else {
            false
        }
    };

    // --- Callback for number of groups input ---
    let set_num_groups = Callback::new(move |num_groups: u32| {
        tournament_editor_context
            .set_stage_num_groups
            .set(num_groups);
    });

    // --- Server Resources & Actions  ---
    // load stage resource
    let stage_res = Resource::new(
        move || (tournament_id(), stage_number()),
        move |(maybe_t_id, maybe_sn)| async move {
            if let Some(t_id) = maybe_t_id
                && let Some(sn) = maybe_sn
            {
                load_stage_by_number(t_id, sn).await
            } else {
                Ok(None)
            }
        },
    );

    // retry function for error handling
    let refetch_and_reset = Callback::new(move |()| {
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
                {move || {
                    stage_res
                        .and_then(|may_be_s| {
                            match may_be_s {
                                Some(stage) => {
                                    tournament_editor_context.set_stage(*stage);
                                }
                                None => {
                                    if let Some(sn) = stage_number() {
                                        tournament_editor_context.new_stage(sn);
                                    }
                                }
                            }
                        })
                }}
                <Show when=move || {
                    matches!(
                        tournament_editor_context.base_mode.get(),
                        Some(TournamentMode::PoolAndFinalStage)
                        | Some(TournamentMode::TwoPoolStagesAndFinalStage)
                    )
                }>
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
                                <fieldset
                                    disabled=move || {
                                        tournament_editor_context.is_busy.get()
                                            || page_err_ctx.has_errors() || is_active_or_done()
                                    }
                                    class="contents"
                                    data-testid="stage-editor-form"
                                >
                                    <div class="w-full max-w-md grid grid-cols-1 gap-6">
                                        <NumberInputWithValidation
                                            label="Number of Groups"
                                            name="stage-num-groups"
                                            value=tournament_editor_context.stage_num_groups
                                            set_value=set_num_groups
                                            validation_result=tournament_editor_context
                                                .validation_result
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
                                                    href=move || url_route_with_sub_path(&i.to_string())
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
            </ErrorBoundary>
        </Transition>
        <Outlet />
    }
}
