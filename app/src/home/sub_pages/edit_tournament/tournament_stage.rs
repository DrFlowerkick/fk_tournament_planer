//! Edit tournament stage component

use app_core::{
    Stage, TournamentBase, TournamentMode, TournamentState, TournamentType,
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
    params::{SportParams, StageParams, TournamentBaseParams},
    server_fn::stage::{SaveStage, load_stage_by_number},
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
    hooks::{use_navigate, use_params, use_query},
    nested_router::Outlet,
};
use uuid::Uuid;

#[component]
pub fn EditTournamentStage() -> impl IntoView {
    // --- Get context for creating and editing tournaments ---
    let tournament_editor = expect_context::<TournamentEditorState>();
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
            tournament_editor.get_tournament().and_then(|t| t.get_id())
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
                tournament_editor.set_stage(stage.clone());
            }
            Some(Ok(None)) => {
                // stage not found, create a new one, if tournament_editor does not have it yet
                let sn = stage_number().unwrap();
                //if tournament_editor.
                // new stage
                let new_state = Stage::new(IdVersion::NewWithId(Uuid::new_v4()), sn);
            }
            _ => (),
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

// Errors to handle
// 1. no tournament_id in query and no tournament in editor state
// 2. no stage_number in params
// 3. invalid stage_number (>= number of stages in tournament)
// 4. stage is active or done (cannot edit active or done stages)
