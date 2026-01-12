//! listing tournaments

use app_core::{TournamentBase, TournamentState, TournamentType};
use app_utils::{params::SportParams, server_fn::tournament_base::list_tournament_bases};
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use uuid::Uuid;

#[component]
pub fn ListTournaments() -> impl IntoView {
    let sport_id_query = use_query::<SportParams>();

    // Signals for Filters
    let (status, set_status) = signal(TournamentState::Scheduling);
    let (include_adhoc, set_include_adhoc) = signal(false);
    let (search_term, set_search_term) = signal("".to_string());
    let (limit, set_limit) = signal(10usize);

    // Signal for Selected Row (UI interaction)
    let (selected_id, set_selected_id) = signal::<Option<Uuid>>(None);

    // Derived Query Params
    let sport_id = move || sport_id_query.get().ok().and_then(|p| p.sport_id);

    // Resource that fetches data when filters change
    let tournaments_data = Resource::new(
        move || (sport_id(), search_term.get(), limit.get()),
        move |(maybe_sport_id, term, lim)| async move {
            if let Some(s_id) = maybe_sport_id {
                list_tournament_bases(s_id, term, Some(lim)).await
            } else {
                Ok(vec![])
            }
        },
    );

    let filtered_tournaments = move || {
        let data = tournaments_data
            .get()
            .unwrap_or(Ok(vec![]))
            .unwrap_or_default();
        data.into_iter()
            .filter(|t| {
                // Filter by status
                (match status.get() {
                    TournamentState::ActiveStage(_) => matches!(t.get_tournament_state(), TournamentState::ActiveStage(_)),
                    _ => t.get_tournament_state() == status.get(),
                }) &&
                // Filter by adhoc
                (include_adhoc.get() || !matches!(t.get_tournament_type(), TournamentType::Adhoc))
            })
            .collect::<Vec<TournamentBase>>()
    };

    view! {
        <div
            class="flex flex-col w-full max-w-6xl mx-auto py-8 space-y-6 px-4"
            data-testid="tournaments-list-root"
        >
            <div class="flex flex-col md:flex-row justify-between items-center gap-4">
                <h2 class="text-3xl font-bold">"List Tournaments"</h2>
            </div>

            // --- Filter Bar ---
            <div class="bg-base-200 p-4 rounded-lg flex flex-wrap gap-4 items-end">

                // Status Filter
                <div class="form-control w-full max-w-xs">
                    <label class="label">
                        <span class="label-text">"Status"</span>
                    </label>
                    <select
                        class="select select-bordered"
                        data-testid="filter-status-select"
                        on:change=move |ev| {
                            set_status.set(TournamentState::from(event_target_value(&ev)))
                        }
                        prop:value=move || format!("{}", status.get())
                    >
                        <option value=format!(
                            "{}",
                            TournamentState::Scheduling,
                        )>{format!("{}", TournamentState::Scheduling)}</option>
                        <option value=format!(
                            "{}",
                            TournamentState::Published,
                        )>{format!("{}", TournamentState::Published)}</option>
                        <option value=format!(
                            "{}",
                            TournamentState::ActiveStage(0),
                        )>{format!("{}", TournamentState::ActiveStage(0))}</option>
                        <option value=format!(
                            "{}",
                            TournamentState::Finished,
                        )>{format!("{}", TournamentState::Finished)}</option>
                    </select>
                </div>

                // Text Search
                <div class="form-control w-full max-w-xs">
                    <label class="label">
                        <span class="label-text">"Search Name"</span>
                    </label>
                    <input
                        type="text"
                        placeholder="Type to search..."
                        class="input input-bordered w-full"
                        data-testid="filter-name-search"
                        on:input=move |ev| set_search_term.set(event_target_value(&ev))
                        prop:value=move || search_term.get()
                    />
                </div>

                // Limit Selector
                <div class="form-control">
                    <label class="label">
                        <span class="label-text">"Limit"</span>
                    </label>
                    <select
                        class="select select-bordered"
                        data-testid="filter-limit-select"
                        on:change=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                set_limit.set(val);
                            }
                        }
                        prop:value=move || limit.get().to_string()
                    >
                        <option value="10">"10"</option>
                        <option value="25">"25"</option>
                        <option value="50">"50"</option>
                    </select>
                </div>

                // Adhoc Toggle
                <div class="form-control">
                    <label class="label cursor-pointer gap-2">
                        <span class="label-text">"Include Adhoc"</span>
                        <input
                            type="checkbox"
                            class="toggle"
                            data-testid="filter-include-adhoc-toggle"
                            on:change=move |ev| set_include_adhoc.set(event_target_checked(&ev))
                            prop:checked=move || include_adhoc.get()
                        />
                    </label>
                </div>
            </div>

            // --- Table Area ---
            <div class="overflow-x-auto">
                <Suspense fallback=move || {
                    view! { <span class="loading loading-spinner loading-lg"></span> }
                }>
                    {move || {
                        let data = filtered_tournaments();
                        if data.is_empty() {

                            view! {
                                <div
                                    class="text-center py-10 bg-base-100 border border-base-300 rounded-lg"
                                    data-testid="tournaments-list-empty"
                                >
                                    <p class="text-lg opacity-60">
                                        "No tournaments found matching your criteria."
                                    </p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <table class="table w-full" data-testid="tournaments-table">
                                    <thead data-testid="tournaments-table-header">
                                        <tr>
                                            <th>"Name"</th>
                                            <th>"Status"</th>
                                            <th>"Entrants"</th>
                                            <th>"Type"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || data.clone()
                                            key=|t| t.get_id().unwrap_or_default()
                                            // Assuming 't' is type TournamentListItem
                                            children=move |t| {
                                                let t_id = t.get_id().unwrap_or_default();
                                                let row_id = t.get_id().unwrap_or_default();
                                                let is_selected = move || {
                                                    selected_id.get() == Some(t_id.clone())
                                                };
                                                let t_render_actions = t.clone();
                                                let render_actions = move || {

                                                    // Action Buttons Helper
                                                    view! {
                                                        <div
                                                            class="flex gap-2 justify-end p-2 bg-base-200"
                                                            data-testid="row-actions"
                                                        >
                                                            // Example Logic based on status
                                                            {match t_render_actions.get_tournament_state() {
                                                                TournamentState::Scheduling | TournamentState::Published => {
                                                                    view! {
                                                                        <button
                                                                            class="btn btn-sm btn-primary"
                                                                            data-testid="action-btn-register"
                                                                        >
                                                                            "Register"
                                                                        </button>
                                                                        <button
                                                                            class="btn btn-sm btn-ghost"
                                                                            data-testid="action-btn-edit"
                                                                        >
                                                                            "Edit"
                                                                        </button>
                                                                        <button class="btn btn-sm" data-testid="action-btn-show">
                                                                            "Show"
                                                                        </button>
                                                                    }
                                                                        .into_any()
                                                                }
                                                                TournamentState::ActiveStage(_) => {
                                                                    view! {
                                                                        <button class="btn btn-sm" data-testid="action-btn-show">
                                                                            "Show"
                                                                        </button>
                                                                    }
                                                                        .into_any()
                                                                }
                                                                TournamentState::Finished => {
                                                                    view! {
                                                                        <button
                                                                            class="btn btn-sm btn-secondary"
                                                                            data-testid="action-btn-results"
                                                                        >
                                                                            "Results"
                                                                        </button>
                                                                    }
                                                                        .into_any()
                                                                }
                                                            }}
                                                        </div>
                                                    }
                                                };

                                                view! {
                                                    <tr
                                                        class="hover cursor-pointer"
                                                        class:bg-base-200=is_selected
                                                        data-testid=format!("tournaments-row-{}", row_id)
                                                        on:click=move |_| {
                                                            if selected_id.get() == Some(row_id.clone()) {
                                                                set_selected_id.set(None);
                                                            } else {
                                                                set_selected_id.set(Some(row_id.clone()));
                                                            }
                                                        }
                                                    >
                                                        <td class="font-bold">{t.get_name().to_string()}</td>
                                                        <td>
                                                            <div class="badge badge-outline">
                                                                {t.get_tournament_state().to_string()}
                                                            </div>
                                                        </td>
                                                        <td>{t.get_num_entrants()}</td>
                                                        <td>{t.get_tournament_mode().to_string()}</td>
                                                    </tr>
                                                    <Show when=is_selected>
                                                        <tr>
                                                            <td colspan="4" class="p-0">
                                                                {render_actions()}
                                                            </td>
                                                        </tr>
                                                    </Show>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            }
                                .into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
