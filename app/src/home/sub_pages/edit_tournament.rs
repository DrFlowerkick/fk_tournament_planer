//! create or edit a tournament

use app_core::{
    TournamentBase, TournamentMode, TournamentState, TournamentType,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use app_utils::{
    error::AppError,
    hooks::{
        is_field_valid::is_field_valid,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{SportParams, TournamentBaseParams},
    server_fn::tournament_base::{SaveTournamentBase, load_tournament_base, save_tournament_base},
    state::{
        global_state::{GlobalState, GlobalStateStoreFields},
        tournament_editor_state::TournamentEditorState,
    },
};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
};
use reactive_stores::Store;
use uuid::Uuid;

#[component]
pub fn EditTournament() -> impl IntoView {
    // --- Initialize context for creating and editing tournaments ---
    let tournament_editor_state = RwSignal::new(TournamentEditorState::new());
    provide_context::<RwSignal<TournamentEditorState>>(tournament_editor_state);

    let is_changed = move || tournament_editor_state.read().is_changed();

    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        nav_url, update, ..
    } = use_query_navigation();

    let sport_id_query = use_query::<SportParams>();
    let tournament_id_query = use_query::<TournamentBaseParams>();

    let global_state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = global_state.sport_plugin_manager();

    // Derived Query Params
    let sport_id = move || sport_id_query.get().ok().and_then(|p| p.sport_id);
    let is_sport_id_error = move || {
        if let Some(sport_id) = sport_id() {
            sport_plugin_manager.get().get_web_ui(&sport_id).is_none()
        } else {
            true
        }
    };
    let tournament_id = move || tournament_id_query.get().ok().and_then(|p| p.tournament_id);

    // Form Signals
    let set_id_version = RwSignal::new(IdVersion::New);
    let set_name = RwSignal::new("".to_string());
    let set_entrants = RwSignal::new(0_u32);
    let set_t_type = RwSignal::new(TournamentType::Scheduled);
    let set_mode = RwSignal::new(TournamentMode::SingleStage);
    let set_num_rounds_swiss = RwSignal::new(0_u32);
    let set_state = RwSignal::new(TournamentState::Draft);

    Effect::new(move || {
        if let TournamentMode::SwissSystem { .. } = set_mode.get_untracked() {
            // keep num_rounds in sync
            set_mode.set(TournamentMode::SwissSystem {
                num_rounds: set_num_rounds_swiss.get(),
            });
        }
    });

    // --- Server Actions & Resources ---
    let save_tournament_base = ServerAction::<SaveTournamentBase>::new();

    let is_save_conflict = move || {
        if let Some(Err(AppError::Core(ce))) = save_tournament_base.value().get()
            && ce.is_optimistic_lock_conflict()
        {
            true
        } else {
            false
        }
    };
    let is_save_duplicate = move || {
        if let Some(Err(AppError::Core(ce))) = save_tournament_base.value().get()
            && ce.is_unique_violation()
        {
            true
        } else {
            false
        }
    };
    let is_general_save_error = move || {
        if let Some(Err(err)) = save_tournament_base.value().get() {
            match err {
                AppError::Core(ce) => {
                    if ce.is_optimistic_lock_conflict() || ce.is_unique_violation() {
                        None
                    } else {
                        Some(format!("{:?}", ce))
                    }
                }
                _ => Some(format!("{:?}", err)),
            }
        } else {
            None
        }
    };

    Effect::new(move || {
        if let Some(Ok(tb)) = save_tournament_base.value().get() {
            save_tournament_base.clear();
            update(
                "tournament_id",
                &tb.get_id().map(|id| id.to_string()).unwrap_or_default(),
            );
            let navigate = use_navigate();
            navigate(
                &nav_url.get(),
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    let tournament_res = Resource::new(
        move || tournament_id(),
        move |t_id| async move {
            if let Some(id) = t_id {
                match load_tournament_base(id).await {
                    Ok(Some(tournament)) => Ok(tournament),
                    Ok(None) => Err(AppError::ResourceNotFound(
                        "Tournament Base".to_string(),
                        id,
                    )),
                    Err(e) => Err(e),
                }
            } else {
                let mut tournament = TournamentBase::default();
                tournament
                    .set_sport_id(sport_id().unwrap_or_default())
                    .set_id_version(IdVersion::NewWithId(Uuid::new_v4()));

                Ok(tournament)
            }
        },
    );

    let is_loading = move || tournament_res.get().is_none();
    let is_pending = save_tournament_base.pending();
    let is_new = move || tournament_id().is_none();
    let is_general_load_error = move || {
        if let Some(Err(err)) = tournament_res.get() {
            Some(format!("{:?}", err))
        } else {
            None
        }
    };

    Effect::new(move || {
        if let Some(Ok(tournament)) = tournament_res.get() {
            set_id_version.set(tournament.get_id_version());
            set_name.set(tournament.get_name().to_string());
            set_entrants.set(tournament.get_num_entrants());
            set_t_type.set(tournament.get_tournament_type());
            set_mode.set(tournament.get_tournament_mode());
            if let TournamentMode::SwissSystem { num_rounds } = tournament.get_tournament_mode() {
                set_num_rounds_swiss.set(num_rounds);
            }
            set_state.set(tournament.get_tournament_state());

            tournament_editor_state.update(|state| {
                state.set_tournament(tournament.clone(), !is_new());
            });
        }
    });

    let refetch_and_reset = move || {
        save_tournament_base.clear();
        tournament_res.refetch();
    };

    // --- Disabled Logic ---
    let is_disabled = move || {
        is_loading()
            || is_pending.get()
            || is_sport_id_error()
            || is_save_conflict()
            || is_save_duplicate()
            || is_general_save_error().is_some()
            || is_general_load_error().is_some()
    };

    // --- Validation Logic ---
    let current_tournament_base = move || {
        let mut tb = TournamentBase::default();
        tb.set_id_version(set_id_version.get())
            .set_name(set_name.get())
            .set_sport_id(sport_id().unwrap_or_default())
            .set_num_entrants(set_entrants.get())
            .set_tournament_type(set_t_type.get())
            .set_tournament_mode(set_mode.get())
            .set_tournament_state(set_state.get());
        tb
    };

    let validation_result = move || current_tournament_base().validate();
    let is_valid_tournament = move || validation_result().is_ok();

    // update tournament editor state when valid tournament changes
    Effect::new(move || {
        if is_valid_tournament() {
            tournament_editor_state.update(|state| {
                state.set_tournament(current_tournament_base(), false);
            });
        }
    });

    let is_valid_name = Signal::derive(move || is_field_valid(validation_result).run("name"));
    let is_valid_entrants =
        Signal::derive(move || is_field_valid(validation_result).run("num_entrants"));
    let is_valid_num_rounds_swiss = Signal::derive(move || {
        if let TournamentMode::SwissSystem { .. } = set_mode.get() {
            is_field_valid(validation_result).run("mode.num_rounds")
        } else {
            None
        }
    });

    // --- Event Handlers ---
    let on_save = move || {
        if let Some(tournament) = tournament_editor_state.read().get_tournament_diff() {
            save_tournament_base.dispatch(SaveTournamentBase { tournament });
        }
        for changed_stage in tournament_editor_state.read().get_stages_diff() {
            // ToDo: save stages
        }
    };

    let on_cancel = move || {
        let navigate = use_navigate();
        let _ = navigate(
            &format!("/?sport_id={}", sport_id().unwrap_or_default()),
            Default::default(),
        );
    };

    view! {
        <div
            class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6"
            data-testid="new-tournament-root"
        >
            <div class="w-full flex justify-between items-center border-b pb-4">
                <h2 class="text-3xl font-bold">
                    {move || if is_new() { "Plan New Tournament" } else { "Edit Tournament" }}
                </h2>
            </div>

        // ToDo: create inputs and buttons.
        </div>
    }
}
