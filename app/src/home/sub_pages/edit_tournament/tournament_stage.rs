//! Edit tournament stage component

use app_core::{
    Stage, TournamentMode, TournamentState,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use app_utils::{
    components::inputs::ValidatedNumberInput,
    error::AppError,
    error::strategy::{handle_general_error, handle_read_error},
    hooks::{
        is_field_valid::is_field_valid,
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{StageParams, TournamentBaseParams},
    server_fn::stage::load_stage_by_number,
    state::{error_state::PageErrorContext, tournament_editor::context::TournamentEditorContext},
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
            tournament_editor_context
                .get_tournament()
                .and_then(|t| t.get_id())
        }
    };

    let stage_number_params = use_params::<StageParams>();
    let stage_number = move || {
        if let Ok(snp) = stage_number_params.get()
            && let Some(sn) = snp.stage_number
            && let Some(tournament) = tournament_editor_context.get_tournament()
            && tournament.get_tournament_mode().get_num_of_stages() > sn
        {
            return Some(sn);
        }
        None
    };
    let editor_title = move || {
        if let Some(sn) = stage_number()
            && let Some(tournament) = tournament_editor_context.get_tournament()
            && let Some(title) = tournament.get_tournament_mode().get_stage_name(sn)
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
            && let Some(tournament) = tournament_editor_context.get_tournament()
            && tournament.get_tournament_mode().get_num_of_stages() <= sn
        {
            // invalid stage number -> signal url validation
            tournament_editor_context.trigger_url_validation();
        }
    });

    // hide form if tournament mode has only one stage with one group OR if invalid stage number
    let hide_form = move || {
        if let Some(tournament) = tournament_editor_context.get_tournament() {
            matches!(
                tournament.get_tournament_mode(),
                TournamentMode::SingleStage | TournamentMode::SwissSystem { num_rounds: _ }
            ) || stage_number().is_none()
        } else {
            false
        }
    };

    // check if stage is not editable
    let is_active_or_done = move || {
        if let Some(tournament) = tournament_editor_context.get_tournament()
            && let Some(sn) = stage_number()
        {
            match tournament.get_tournament_state() {
                TournamentState::ActiveStage(acs) => acs >= sn,
                TournamentState::Finished => true,
                _ => false,
            }
        } else {
            false
        }
    };

    // Form Signals
    let set_id_version = RwSignal::new(IdVersion::New);
    let set_num_groups = RwSignal::new(1u32);

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

    // --- Validation Logic ---
    let current_stage = Memo::new(move |_| {
        if let Some(sn) = stage_number()
            && let Some(mut stage) = tournament_editor_context.get_stage_by_number_untracked(sn)
        {
            stage.set_num_groups(set_num_groups.get());
            Some(stage)
        } else {
            None
        }
    });

    // Sync to Global State
    Effect::new(move || {
        if let Some(stage) = current_stage.get() {
            tournament_editor_context.set_stage(stage.clone(), false);
        }
    });

    // Validation runs against the constantly updated Memo
    let validation_result = Signal::derive(move || {
        if let Some(tournament) = tournament_editor_context.get_tournament()
            && let Some(current_stage) = current_stage.get()
        {
            current_stage.validate(&tournament)
        } else {
            Ok(())
        }
    });

    // error messages for form fields
    let num_groups_error = Signal::derive(move || is_field_valid(validation_result, "num_groups"));

    view! {
        <Show when=move || !hide_form()>
            <div
                class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6"
                data-testid="stage-editor-root"
            >
                <div class="w-full flex justify-between items-center pb-4">
                    <h2 class="text-3xl font-bold" data-testid="stage-editor-title">
                        {move || editor_title()}
                    </h2>
                </div>

                // Card wrapping Form and Group Links
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
                                {move || {
                                    stage_res
                                        .and_then(|may_be_s| {
                                            match may_be_s {
                                                Some(stage) => {
                                                    tournament_editor_context.set_stage(*stage, true);
                                                    set_id_version.set(stage.get_id_version());
                                                    set_num_groups.set(stage.get_num_groups());
                                                }
                                                None => {
                                                    if let Some(t_id) = tournament_id()
                                                        && let Some(sn) = stage_number()
                                                    {
                                                        if let Some(stage) = tournament_editor_context
                                                            .get_stage_by_number(sn)
                                                        {
                                                            set_id_version.set(stage.get_id_version());
                                                            set_num_groups.set(stage.get_num_groups());
                                                        } else {
                                                            let mut new_state = Stage::new(
                                                                IdVersion::NewWithId(Uuid::new_v4()),
                                                            );
                                                            new_state.set_number(sn);
                                                            new_state.set_tournament_id(t_id);
                                                            tournament_editor_context.set_stage(new_state, false);
                                                            set_id_version.set(new_state.get_id_version());
                                                            set_num_groups.set(new_state.get_num_groups());
                                                        }
                                                    }
                                                }
                                            }
                                        })
                                }}
                                <fieldset
                                    disabled=move || {
                                        tournament_editor_context.is_busy()
                                            || page_err_ctx.has_errors() || is_active_or_done()
                                    }
                                    class="contents"
                                    data-testid="stage-editor-form"
                                >
                                    <div class="w-full max-w-md grid grid-cols-1 gap-6">
                                        <ValidatedNumberInput
                                            label="Number of Groups"
                                            name="stage-num-groups"
                                            value=set_num_groups
                                            validation_error=num_groups_error
                                            min="1".to_string()
                                        />
                                    </div>
                                // group editor links
                                </fieldset>
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full mt-6">
                                    <For
                                        each=move || 0..set_num_groups.get()
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
                            </ErrorBoundary>
                        </Transition>
                    </div>
                </div>
            </div>
        </Show>
        <Outlet />
    }
}
