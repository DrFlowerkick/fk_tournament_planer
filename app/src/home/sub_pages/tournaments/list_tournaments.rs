//! list tournaments

use app_core::{TournamentBase, TournamentState, TournamentType};
use app_utils::{
    components::inputs::EnumSelectWithValidation,
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
    params::use_sport_id_query,
    server_fn::tournament_base::list_tournament_bases,
    state::error_state::PageErrorContext,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, components::A, hooks::use_navigate, nested_router::Outlet};
use uuid::Uuid;

#[component]
pub fn ListTournaments() -> impl IntoView {
    // navigation and query handling Hook
    let UseQueryNavigationReturn {
        url_matched_route_update_query,
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    // Signals for Filters
    let (status, set_status) = signal(Some(TournamentState::Draft));
    let set_status = Callback::new(move |new_status: Option<TournamentState>| {
        set_status.set(new_status);
    });
    let (include_adhoc, set_include_adhoc) = signal(false);
    let (search_term, set_search_term) = signal("".to_string());
    let (limit, set_limit) = signal(10usize);

    // Signal for Selected Row (UI interaction)
    let (selected_id, set_selected_id) = signal::<Option<Uuid>>(None);

    // update tournament_id query param when selected_id changes
    Effect::new({
        let navigate = navigate.clone();
        move || {
            let nav_url = if let Some(t_id) = selected_id.get() {
                url_matched_route_update_query(
                    "tournament_id",
                    &t_id.to_string(),
                    MatchedRouteHandler::Keep,
                )
            } else {
                url_matched_route_remove_query("tournament_id", MatchedRouteHandler::Keep)
            };
            navigate(
                &nav_url,
                NavigateOptions {
                    replace: true,
                    scroll: false,
                    ..Default::default()
                },
            );
        }
    });

    // Derived Query Params
    let sport_id = use_sport_id_query();

    // Resource that fetches data when filters change
    let tournaments_data = Resource::new(
        move || {
            (
                sport_id.get(),
                search_term.get(),
                limit.get(),
                status.get(),
                include_adhoc.get(),
            )
        },
        move |(maybe_sport_id, term, lim, status, include_adhoc)| async move {
            if let Some(s_id) = maybe_sport_id {
                list_tournament_bases(s_id, term, Some(lim))
                    .await
                    .map(|tournaments| {
                        tournaments
                            .into_iter()
                            .filter(|t| {
                                // Filter by status
                                (match status {
                                    Some(TournamentState::ActiveStage(_)) => matches!(t.get_tournament_state(), TournamentState::ActiveStage(_)),
                                    Some(s) => Some(s) == Some(t.get_tournament_state()),
                                    None => true,
                                }) &&
                                // Filter by adhoc
                                (include_adhoc || !matches!(t.get_tournament_type(), TournamentType::Adhoc))
                            })
                            .collect::<Vec<TournamentBase>>()
                    })
            } else {
                Ok(vec![])
            }
        },
    );

    // Refetch function for errors
    let refetch = Callback::new(move |()| tournaments_data.refetch());

    // on_cancel handler
    let on_cancel = use_on_cancel();

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
                    <EnumSelectWithValidation
                        label="Filter Tournament State"
                        name="filter-tournament-state"
                        value=status
                        set_value=set_status
                        clear_label="No Status Filter"
                    />
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
                <Transition fallback=move || {
                    view! { <span class="loading loading-spinner loading-lg"></span> }
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
                            tournaments_data
                                .and_then(|data| {
                                    if let Some(selected_id) = selected_id.get_untracked()
                                        && !data.iter().any(|t| t.get_id() == selected_id)
                                    {
                                        set_selected_id.set(None);
                                    }
                                    let data = StoredValue::new(data.clone());
                                    view! {
                                        <Show
                                            when=move || !data.read_value().is_empty()
                                            fallback=|| {
                                                view! {
                                                    <div
                                                        class="text-center py-10 bg-base-100 border border-base-300 rounded-lg"
                                                        data-testid="tournaments-list-empty"
                                                    >
                                                        <p class="text-lg opacity-60">
                                                            "No tournaments found with the current filters."
                                                        </p>
                                                    </div>
                                                }
                                            }
                                        >
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
                                                        each=move || data.read_value().clone()
                                                        key=|t| t.get_id()
                                                        children=move |t| {
                                                            let t_id = t.get_id();
                                                            let is_selected = move || {
                                                                selected_id.get() == Some(t_id)
                                                            };

                                                            view! {
                                                                <tr
                                                                    class="hover cursor-pointer"
                                                                    class:bg-base-200=is_selected
                                                                    data-testid=format!("tournaments-row-{}", t_id)
                                                                    on:click=move |_| {
                                                                        if selected_id.get() == Some(t_id) {
                                                                            set_selected_id.set(None);
                                                                        } else {
                                                                            set_selected_id.set(Some(t_id));
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
                                                                            <SelectedTournamentActions tournament_state=t
                                                                                .get_tournament_state() />
                                                                        </td>
                                                                    </tr>
                                                                </Show>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </Show>
                                    }
                                })
                        }}
                    </ErrorBoundary>
                </Transition>
            </div>
        </div>
        <Outlet />
    }
}

#[component]
pub fn SelectedTournamentActions(tournament_state: TournamentState) -> impl IntoView {
    // navigation and query handling Hook
    let UseQueryNavigationReturn {
        url_matched_route, ..
    } = use_query_navigation();

    view! {
        <div class="flex gap-2 justify-end p-2 bg-base-200" data-testid="row-actions">
            // Example Logic based on status
            {match tournament_state {
                TournamentState::Draft | TournamentState::Published => {
                    view! {
                        <A
                            href=move || url_matched_route(MatchedRouteHandler::Extend("register"))
                            attr:class="btn btn-sm btn-primary"
                            attr:data-testid="action-btn-register"
                            scroll=false
                        >
                            "Register"
                        </A>
                        <A
                            href=move || {
                                let url =
                                url_matched_route(MatchedRouteHandler::Extend("edit"));
                                leptos::logging::log!("Edit Tournament URL: {}", url);
                                url
                            }
                            attr:class="btn btn-sm btn-ghost"
                            attr:data-testid="action-btn-edit"
                            scroll=false
                        >
                            "Edit"
                        </A>
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
                        <button class="btn btn-sm btn-secondary" data-testid="action-btn-results">
                            "Results"
                        </button>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}
