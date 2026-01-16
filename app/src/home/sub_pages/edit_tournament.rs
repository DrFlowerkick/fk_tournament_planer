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
        error_state::PageErrorContext, toast_state::ToastContext,
        tournament_editor_state::TournamentEditorState,
    },
};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
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
        nav_url, update, ..
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

    // Derived Query Params
    // sport id is save to unwrap here because home page ensures valid sport id before rendering this component
    let sport_id = move || {
        sport_id_query
            .get()
            .ok()
            .and_then(|p| p.sport_id)
            .unwrap_or_default()
    };
    let tournament_id = move || tournament_id_query.get().ok().and_then(|p| p.tournament_id);
    let is_new = Signal::derive(move || tournament_id().is_none());

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

    // --- Server Resources & Actions  ---
    // load tournament base resource
    let tournament_res = Resource::new(
        move || (tournament_id(), sport_id()), // Fix: sport_id() hier aufrufen
        move |(t_id, s_id)| async move {
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
                    .set_sport_id(s_id)
                    .set_id_version(IdVersion::NewWithId(Uuid::new_v4()));

                Ok(tournament)
            }
        },
    );

    // handle successful load
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
            let _ = navigate(&format!("/?sport_id={}", sport_id()), Default::default());
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

    // --- Disabled Logic ---
    let is_input_disabled = move || is_pending.get() || page_err_ctx.has_errors();

    // --- Validation Logic ---
    // We make this a Memo so that validation doesn't run on every render cycle,
    // but only when one of the signals changes.
    let current_tournament_base = Memo::new(move |_| {
        let mut tb = TournamentBase::default();
        tb.set_id_version(set_id_version.get())
            .set_name(set_name.get())
            .set_sport_id(sport_id())
            .set_num_entrants(set_entrants.get())
            .set_tournament_type(set_t_type.get())
            .set_tournament_mode(set_mode.get())
            .set_tournament_state(set_state.get());
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
            // Test ID helps with integration tests
            data-testid="tournament-editor-root"
        >
            <div class="w-full flex justify-between items-center border-b pb-4">
                <h2 class="text-3xl font-bold">
                    {move || if is_new.get() { "Plan New Tournament" } else { "Edit Tournament" }}
                </h2>
            // Header Actions (optional, if you want buttons at the top too)
            </div>

            // --- Form Area ---
            <Transition fallback=move || {
                view! {
                    <div class="w-full flex justify-center py-8">
                        <span class="loading loading-spinner loading-lg"></span>
                    </div>
                }
            }>
                <fieldset disabled=is_input_disabled class="contents">
                    <div class="w-full grid grid-cols-1 md:grid-cols-2 gap-6">

                        // 1. Name
                        <ValidatedTextInput
                            label="Tournament Name"
                            name="name"
                            value=set_name
                            error_message=name_error
                            is_new=is_new
                        />

                        // 2. Entrants
                        <ValidatedNumberInput
                            label="Number of Entrants"
                            name="num_entrants"
                            value=set_entrants
                            error_message=entrants_error
                            is_new=is_new
                            min="2".to_string()
                        />

                        // 3. Tournament Type (Enum Select)
                        <EnumSelect label="Tournament Type" name="type" value=set_t_type />

                        // 4. Tournament Mode (Enum Select)
                        <EnumSelect label="Mode" name="mode" value=set_mode />

                        // 5. Conditional Input: Swiss Rounds
                        <Show when=move || {
                            matches!(set_mode.get(), TournamentMode::SwissSystem { .. })
                        }>
                            <ValidatedNumberInput
                                label="Rounds (Swiss System)"
                                name="num_rounds"
                                value=set_num_rounds_swiss
                                error_message=rounds_error
                                is_new=is_new
                                min="1".to_string()
                            />
                        </Show>

                        // 6. State (Enum Select)
                        <EnumSelect label="Status" name="state" value=set_state />
                    </div>
                </fieldset>
            </Transition>

            // --- Action Buttons ---
            <div class="w-full flex justify-end gap-4 pt-6 border-t">
                <button
                    class="btn btn-ghost"
                    on:click=move |_| on_cancel().clone()
                    // Disable cancel only on save, not load
                    disabled=move || is_pending.get()
                >
                    "Cancel"
                </button>

                <button
                    class="btn btn-primary"
                    on:click=move |_| on_save()
                    // Disabled when: Loading, Saving, Validation failed OR nothing changed
                    disabled=move || is_input_disabled() || !is_valid_tournament() || !is_changed()
                >
                    {move || {
                        if is_pending.get() {
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
