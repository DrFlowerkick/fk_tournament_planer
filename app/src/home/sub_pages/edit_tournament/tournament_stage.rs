//! Edit tournament stage component

use app_core::{
    Stage, TournamentBase, TournamentMode, TournamentState, TournamentType,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use app_utils::{
    components::inputs::{EnumSelect, ValidatedNumberInput, ValidatedTextInput},
    error::{
        AppError,
        strategy::{handle_general_error, handle_read_error, handle_write_error},
    },
    hooks::{
        is_field_valid::is_field_valid,
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{SportParams, StageParams, TournamentBaseParams},
    server_fn::stage::load_stage_by_number,
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
pub fn EditTournamentStage() -> impl IntoView {
    // --- Get context for creating and editing tournaments ---
    let tournament_editor_context = expect_context::<TournamentEditorContext>();
    let page_err_ctx = expect_context::<PageErrorContext>();
    let toast_ctx = expect_context::<ToastContext>();
    let component_id = StoredValue::new(Uuid::new_v4());

    // --- Hooks & Navigation ---
    let UseQueryNavigationReturn {
        nav_url,
        update,
        relative_sub_url,
        path,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
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
        stage_number_params
            .get()
            .ok()
            .and_then(|snp| snp.stage_number)
    };
    let has_required_inputs = move || tournament_id().is_some() && stage_number().is_some();

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

    // handle successful load
    Effect::new(move || {
        if !has_required_inputs() {
            return;
        }
        match stage_res.get() {
            Some(Ok(Some(stage))) => {
                // stage successfully loaded
                set_id_version.set(stage.get_id_version());
                set_num_groups.set(stage.get_num_groups());
                tournament_editor_context.set_stage(stage, true);
            }
            Some(Ok(None)) => {
                // stage not found, create a new one, if tournament_editor_context does not have it yet
                let sn = stage_number().unwrap();
                if tournament_editor_context.get_stage_by_number(sn).is_none() {
                    // new stage
                    let mut new_state = Stage::new(IdVersion::NewWithId(Uuid::new_v4()));
                    new_state.set_number(sn);
                    set_id_version.set(new_state.get_id_version());
                    set_num_groups.set(new_state.get_num_groups());
                    tournament_editor_context.set_stage(new_state, false);
                }
            }
            _ => (),
        }
    });

    // --- handle errors ---
    // retry function for error handling
    let refetch_and_reset = move || {
        stage_res.refetch();
    };

    // cancel function for cancel button and error handling
    let on_cancel = use_on_cancel();

    // Handle read errors
    Effect::new({
        let on_cancel = on_cancel.clone();
        move || {
            if let Some(Err(err)) = stage_res.get() {
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

    // handle input errors
    Effect::new({
        let on_cancel = on_cancel.clone();
        move || {
            if tournament_id().is_none() {
                handle_general_error(
                    &page_err_ctx,
                    component_id.get_value(),
                    "Stage Editing requires Tournament ID.",
                    Some(refetch_and_reset.clone()),
                    on_cancel.clone(),
                );
            }
        }
    });
    Effect::new({
        let on_cancel = on_cancel.clone();
        move || {
            if stage_number().is_none() {
                handle_general_error(
                    &page_err_ctx,
                    component_id.get_value(),
                    "Stage Editing requires Stage Number.",
                    Some(refetch_and_reset.clone()),
                    on_cancel.clone(),
                );
            }
        }
    });
    // handle active or done stage error
    Effect::new({
        let on_cancel = on_cancel.clone();
        move || {
            if let Some(tournament) = tournament_editor_context.get_tournament()
                && let Some(sn) = stage_number()
            {
                let error_msg = match tournament.get_tournament_state() {
                    TournamentState::ActiveStage(acs) if acs == sn => {
                        "Stage to edit is currently active and cannot be edited."
                    }
                    TournamentState::Finished => "Tournament is finished; stages cannot be edited.",
                    _ => return,
                };
                handle_general_error(
                    &page_err_ctx,
                    component_id.get_value(),
                    error_msg,
                    Some(refetch_and_reset.clone()),
                    on_cancel.clone(),
                );
            }
        }
    });

    // --- Validation Logic ---
    let current_stage = Memo::new(move |_| {
        let mut stage = Stage::new(set_id_version.get());
        if let Some(tournament_id) = tournament_id() {
            stage.set_tournament_id(tournament_id);
        }
        if let Some(sn) = stage_number() {
            stage.set_number(sn);
        }
        stage.set_num_groups(set_num_groups.get());
        stage
    });

    // Validation runs against the constantly updated Memo
    let validation_result = move || {
        if let Some(tournament) = tournament_editor_context.get_tournament() {
            current_stage.get().validate(&tournament)
        } else {
            Ok(())
        }
    };
    let is_valid_stage = move || validation_result().is_ok();

    // Sync to Global State: Only if valid!
    Effect::new(move || {
        if is_valid_stage() {
            tournament_editor_context.set_stage(current_stage.get(), false);
        }
    });

    // Helper for error messages in the inputs
    // derive creates a read-only signal for the inputs
    let num_groups_error =
        Signal::derive(move || is_field_valid(validation_result).run("num_groups"));
    // validation checks valid stage number, but since stage number is from params and not editable here,
    // we show the error related to entrants here
    Effect::new({
        let on_cancel = on_cancel.clone();
        move || {
            if let Some(error_msg) = is_field_valid(validation_result).run("number") {
                handle_general_error(
                    &page_err_ctx,
                    component_id.get_value(),
                    error_msg,
                    Some(refetch_and_reset.clone()),
                    on_cancel.clone(),
                );
            }
        }
    });

    view! {
        <div class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6">
            <h2 class="text-3xl font-bold">"Edit Tournament Stage"</h2>
            <p class="text-base-content/70 text-center">
                "ToDo: Add information about editing a tournament stage."
            </p>
        </div>
        <Outlet />
    }
}
