//! create or edit a tournament

use app_core::{TournamentBase, TournamentMode, TournamentState, TournamentType};
use app_utils::{
    error::AppError,
    hooks::{
        is_field_valid::is_field_valid,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::{SportParams, TournamentBaseParams},
    server_fn::tournament_base::{SaveTournamentBase, load_tournament_base, save_tournament_base},
};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
};
use uuid::Uuid;

#[component]
pub fn EditTournament() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        nav_url,
        update,
        path,
        query_string,
        ..
    } = use_query_navigation();

    let sport_id_query = use_query::<SportParams>();
    let tournament_id_query = use_query::<TournamentBaseParams>();

    // Derived Query Params
    let sport_id = move || sport_id_query.get().ok().and_then(|p| p.sport_id);
    let tournament_id = move || tournament_id_query.get().ok().and_then(|p| p.tournament_id);

    // Form Signals
    let set_name = RwSignal::new("".to_string());
    let set_entrants = RwSignal::new(0_u32);
    let set_t_type = RwSignal::new(TournamentType::Scheduled);
    let set_mode = RwSignal::new(TournamentMode::SingleStage);
    let set_num_rounds_swiss = RwSignal::new(0_u32);
    let set_state = RwSignal::new(TournamentState::Scheduling);

    // We need to track the version for concurrency control (optimistic locking)
    let (version, set_version) = signal(0_u32);

    // --- Server Actions & Resources ---
    let save_tournament_base = ServerAction::<SaveTournamentBase>::new();

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
                    Ok(Some(tournament)) => {
                        set_name.set(tournament.get_name().to_string());
                        set_entrants.set(tournament.get_num_entrants());
                        set_t_type.set(tournament.get_tournament_type());
                        set_mode.set(tournament.get_tournament_mode());
                        if let TournamentMode::SwissSystem { num_rounds } =
                            tournament.get_tournament_mode()
                        {
                            set_num_rounds_swiss.set(num_rounds);
                        }
                        set_state.set(tournament.get_tournament_state());
                        set_version.set(tournament.get_version().unwrap_or_default());
                        Ok(tournament)
                    }
                    Ok(None) => Err(AppError::ResourceNotFound(
                        "Tournament Base".to_string(),
                        id,
                    )),
                    Err(e) => Err(e),
                }
            } else {
                Ok(TournamentBase::default())
            }
        },
    );

    let is_loading = move || tournament_res.get().is_none();
    let is_pending = save_tournament_base.pending();
    let is_new = move || tournament_id().is_none();

    // --- Validation Logic ---
    let current_tournament_base = move || {
        let mut tb = TournamentBase::default();
        tb.set_name(set_name.get())
            .set_num_entrants(set_entrants.get())
            .set_tournament_type(set_t_type.get())
            .set_tournament_mode(set_mode.get())
            .set_tournament_state(set_state.get());
        tb
    };

    let validation_result = move || current_tournament_base().validate();
    let is_valid_addr = move || validation_result().is_ok();

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
