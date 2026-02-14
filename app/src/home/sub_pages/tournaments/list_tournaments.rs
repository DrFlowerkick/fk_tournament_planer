//! list tournaments

use app_core::{CrTopic, TournamentBase, TournamentState, TournamentType};
use app_utils::{
    components::inputs::EnumSelectFilter,
    enum_utils::FilterLimit,
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
    params::{
        FilterLimitQuery, FilterNameQuery, IncludeAdhocQuery, ParamQuery, SportIdQuery,
        TournamentBaseIdQuery, TournamentStateQuery,
    },
    server_fn::tournament_base::list_tournament_bases,
    state::{
        activity_tracker::ActivityTracker, error_state::PageErrorContext,
        object_table_list::ObjectListContext,
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::{html::H2, prelude::*};
use leptos_router::{
    components::{A, Form},
    nested_router::Outlet,
};
use uuid::Uuid;

#[component]
pub fn ListTournaments() -> impl IntoView {
    // navigation and query handling Hook
    let UseQueryNavigationReturn {
        url_is_matched_route,
        ..
    } = use_query_navigation();

    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- local context ---
    // ToDo: may be wo should provide this for edit tournament, but at the moment we do not need this.
    let tournament_list_ctx = ObjectListContext::<TournamentBase, TournamentBaseIdQuery>::new();

    // Signals for Filters
    let sport_id = SportIdQuery::use_param_query();
    let tournament_base_id = TournamentBaseIdQuery::use_param_query();
    let tournament_state = TournamentStateQuery::use_param_query();
    let search_term = FilterNameQuery::use_param_query();
    let limit = FilterLimitQuery::use_param_query();
    let include_adhoc = IncludeAdhocQuery::use_param_query();

    // Resource that fetches data when filters change
    let tournaments_data = Resource::new(
        move || {
            (
                sport_id.get(),
                search_term.get(),
                limit.get(),
                tournament_state.get(),
                include_adhoc.get(),
            )
        },
        move |(maybe_sport_id, term, lim, status, include_adhoc)| async move {
            if let Some(s_id) = maybe_sport_id {
                activity_tracker
                    .track_activity_wrapper(
                        component_id.get_value(),
                        list_tournament_bases(
                            s_id,
                            term.unwrap_or_default(),
                            lim.or_else(|| Some(FilterLimit::default()))
                                .map(|l| l as usize)),
                    )
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
                                (include_adhoc.unwrap_or(false) || !matches!(t.get_tournament_type(), TournamentType::Adhoc))
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

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <div class="card w-full bg-base-100 shadow-xl" data-testid="tournaments-list-root">
            <div class="card-body">
                <h2 class="card-title" node_ref=scroll_ref>
                    "List Tournaments"
                </h2>

                // --- Filter Bar ---
                <Form method="GET" action="" noscroll=true replace=true>
                    // Hidden input to keep sport_id and sport_config_id in query string
                    <input
                        type="hidden"
                        name=SportIdQuery::key()
                        prop:value=move || {
                            sport_id.get().map(|id| id.to_string()).unwrap_or_default()
                        }
                    />
                    <input
                        type="hidden"
                        name=TournamentBaseIdQuery::key()
                        prop:value=move || {
                            tournament_base_id.get().map(|id| id.to_string()).unwrap_or_default()
                        }
                    />
                    <div class="bg-base-200 p-4 rounded-lg flex flex-wrap gap-4 items-end">

                        // Status Filter
                        <div class="w-full max-w-xs">
                            <EnumSelectFilter<
                            TournamentState,
                        >
                                name=TournamentStateQuery::key()
                                label="Tournament State"
                                value=tournament_state
                                data_testid="filter-tournament-state-select"
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
                                name=FilterNameQuery::key()
                                placeholder="Type to search for name..."
                                class="input input-bordered w-full"
                                data-testid="filter-name-search"
                                prop:value=move || search_term.get()
                                oninput="this.form.requestSubmit()"
                            />
                        </div>
                        // Limit Selector
                        <div class="w-full max-w-xs">
                            <EnumSelectFilter<
                            FilterLimit,
                        >
                                name=FilterLimitQuery::key()
                                label="Limit"
                                value=limit
                                data_testid="filter-limit-select"
                                clear_label=FilterLimit::default().to_string()
                            />
                        </div>

                        // Adhoc Toggle
                        <div class="form-control w-full max-w-xs flex flex-col">
                            <label class="label">
                                <span class="label-text">"Include Adhoc"</span>
                            </label>
                            <input
                                type="checkbox"
                                class="toggle"
                                name=IncludeAdhocQuery::key()
                                data-testid="filter-include-adhoc-toggle"
                                value="true"
                                prop:checked=move || include_adhoc.get().unwrap_or(false)
                                oninput="this.form.requestSubmit()"
                            />
                        </div>
                    </div>
                </Form>

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
                                        tournament_list_ctx.object_list.set(data.clone());
                                        view! {
                                            <Show
                                                when=move || {
                                                    tournament_list_ctx.object_list.with(|val| !val.is_empty())
                                                }
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
                                                            <th>"Preview"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <For
                                                            each=move || { tournament_list_ctx.object_list.get() }
                                                            key=|t| t.get_id()
                                                            children=move |t| {
                                                                let t = StoredValue::new(t);
                                                                let is_selected = move || {
                                                                    tournament_list_ctx.selected_id.get()
                                                                        == Some(t.read_value().get_id())
                                                                };
                                                                let topic = Signal::derive(move || {
                                                                    Some(CrTopic::TournamentBase(t.read_value().get_id()))
                                                                });
                                                                let version = Signal::derive({
                                                                    let t = t.clone();
                                                                    move || { t.read_value().get_version().unwrap_or_default() }
                                                                });
                                                                use_client_registry_socket(topic, version, refetch);

                                                                view! {
                                                                    <tr
                                                                        class="hover cursor-pointer"
                                                                        class:bg-base-200=is_selected
                                                                        data-testid=format!(
                                                                            "tournaments-row-{}",
                                                                            t.read_value().get_id(),
                                                                        )
                                                                        on:click=move |_| {
                                                                            if tournament_list_ctx.selected_id.get()
                                                                                == Some(t.read_value().get_id())
                                                                            {
                                                                                tournament_list_ctx.set_selected_id.run(None);
                                                                            } else {
                                                                                tournament_list_ctx
                                                                                    .set_selected_id
                                                                                    .run(Some(t.read_value().get_id()));
                                                                            }
                                                                        }
                                                                    >
                                                                        <td class="font-bold">
                                                                            {t.read_value().get_name().to_string()}
                                                                        </td>
                                                                        <td data-testid=format!(
                                                                            "table-entry-preview-{}",
                                                                            t.read_value().get_id(),
                                                                        )>
                                                                            <p>
                                                                                <span class="badge badge-outline mr-2">
                                                                                    {t.read_value().get_tournament_state().to_string()}
                                                                                </span>
                                                                                {format!(
                                                                                    "{} with {} entrants",
                                                                                    t.read_value().get_tournament_mode(),
                                                                                    t.read_value().get_num_entrants(),
                                                                                )}
                                                                            </p>
                                                                        </td>
                                                                    </tr>
                                                                    <Show when=is_selected>
                                                                        <tr>
                                                                            <td colspan="2" class="p-0">
                                                                                <div
                                                                                    class="p-4 bg-base-100 border border-base-300 rounded-lg"
                                                                                    data-testid="table-entry-detailed-preview"
                                                                                >
                                                                                    <h3 class="font-bold text-lg mb-2">"Tournament Details"</h3>
                                                                                    <p>
                                                                                        <strong>"ID: "</strong>
                                                                                        {t.read_value().get_id().to_string()}
                                                                                    </p>
                                                                                    <p>
                                                                                        <strong>"Type: "</strong>
                                                                                        {t.read_value().get_tournament_type().to_string()}
                                                                                    </p>
                                                                                    <p>
                                                                                        <strong>"State: "</strong>
                                                                                        {t.read_value().get_tournament_state().to_string()}
                                                                                    </p>
                                                                                    <p>
                                                                                        <strong>"Number of Entrants: "</strong>
                                                                                        {t.read_value().get_num_entrants()}
                                                                                    </p>
                                                                                </div>
                                                                            </td>
                                                                        </tr>
                                                                        <tr>
                                                                            <td colspan="2" class="p-0">
                                                                                <SelectedTournamentActions tournament_state=t
                                                                                    .read_value()
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
        </div>
        <div class="my-4"></div>
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
                            href=move || url_matched_route(MatchedRouteHandler::Extend("edit"))
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
