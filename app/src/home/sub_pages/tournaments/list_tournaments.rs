//! list tournaments

use app_core::TournamentState;
use app_utils::{
    components::inputs::{EnumSelect, InputCommitAction, InputUpdateStrategy, TextInput},
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
    server_fn::tournament_base::list_tournament_base_ids,
    state::{
        SimpleEditorOptions, activity_tracker::ActivityTracker, error_state::PageErrorContext,
        object_table::ObjectEditorMapContext, toast_state::ToastContext,
        tournament::TournamentEditorContext,
    },
};
use leptos::{html::H2, prelude::*};
use leptos_router::{
    NavigateOptions,
    components::{A, Form},
    hooks::use_navigate,
    nested_router::Outlet,
};
use uuid::Uuid;

#[component]
pub fn ListTournaments() -> impl IntoView {
    // navigation and query handling Hook
    let UseQueryNavigationReturn {
        url_is_matched_route,
        url_matched_route,
        url_matched_route_update_query,
        ..
    } = use_query_navigation();

    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let toast_ctx = expect_context::<ToastContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- local context ---

    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();

    // Signals for Filters
    let sport_id = SportIdQuery::use_param_query();
    let tournament_base_id = TournamentBaseIdQuery::use_param_query();
    let search_term = FilterNameQuery::use_param_query();
    let tournament_state = TournamentStateQuery::use_param_query();
    let include_adhoc = IncludeAdhocQuery::use_param_query();
    let limit = FilterLimitQuery::use_param_query();

    // Resource that fetches data when filters change
    let tournament_ids = Resource::new(
        move || {
            (
                sport_id.get(),
                search_term.get(),
                tournament_state.get(),
                include_adhoc.get(),
                limit.get(),
            )
        },
        move |(maybe_sport_id, term, status, include_adhoc, lim)| async move {
            if let Some(s_id) = maybe_sport_id {
                activity_tracker
                    .track_activity_wrapper(
                        component_id.get_value(),
                        list_tournament_base_ids(
                            s_id,
                            term.unwrap_or_default(),
                            status,
                            include_adhoc.unwrap_or(false),
                            lim.or_else(|| Some(FilterLimit::default()))
                                .map(|l| l as usize),
                        ),
                    )
                    .await
            } else {
                Ok(vec![])
            }
        },
    );

    // Refetch function for errors
    let refetch = Callback::new(move |()| tournament_ids.refetch());

    // on_cancel handler
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <Transition fallback=move || {
            view! {
                <div class="card w-full bg-base-100 shadow-xl" data-testid="tournaments-list-root">
                    <div class="card-body">
                        <h2 class="card-title" node_ref=scroll_ref>
                            "List Tournaments"
                        </h2>
                        <span class="loading loading-spinner loading-lg"></span>
                    </div>
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
                    tournament_ids
                        .and_then(|t_ids| {
                            tournament_editor_map.visible_ids_list.set(t_ids.clone());
                            view! {
                                <div
                                    class="card w-full bg-base-100 shadow-xl"
                                    data-testid="tournaments-list-root"
                                >
                                    <div class="card-body">
                                        <div class="flex justify-between items-center">
                                            <h2 class="card-title" node_ref=scroll_ref>
                                                "List Tournaments"
                                            </h2>
                                            <button
                                                class="btn btn-square btn-ghost btn-sm"
                                                on:click=move |_| on_cancel.run(())
                                                aria-label="Close"
                                                data-testid="action-btn-close-list"
                                            >
                                                <span class="icon-[heroicons--x-mark] w-6 h-6"></span>
                                            </button>
                                        </div>

                                        // --- Filter Bar ---
                                        <Form method="GET" action="" noscroll=true replace=true>
                                            // Hidden input to keep sport_id and sport_config_id in query string
                                            <input
                                                type="hidden"
                                                name=SportIdQuery::KEY
                                                prop:value=move || {
                                                    sport_id.get().map(|id| id.to_string()).unwrap_or_default()
                                                }
                                            />
                                            <input
                                                type="hidden"
                                                name=TournamentBaseIdQuery::KEY
                                                prop:value=move || {
                                                    tournament_base_id
                                                        .get()
                                                        .map(|id| id.to_string())
                                                        .unwrap_or_default()
                                                }
                                            />
                                            <div class="bg-base-200 p-4 rounded-lg flex flex-wrap gap-4 items-end">

                                                // Status Filter
                                                <div class="w-full max-w-xs">
                                                    <EnumSelect<
                                                    TournamentState,
                                                >
                                                        name=TournamentStateQuery::KEY
                                                        label="Tournament State"
                                                        value=tournament_state
                                                        data_testid="filter-tournament-state-select"
                                                        clear_label="No Status Filter"
                                                        action=InputCommitAction::SubmitForm
                                                    />
                                                </div>

                                                // Text Search
                                                <div class="w-full max-w-xs">
                                                    <TextInput<
                                                    String,
                                                >
                                                        name=FilterNameQuery::KEY
                                                        label="Search Name"
                                                        placeholder="Type to search for name..."
                                                        value=search_term
                                                        update_on=InputUpdateStrategy::Input
                                                        action=InputCommitAction::SubmitForm
                                                        data_testid="filter-name-search"
                                                    />
                                                </div>
                                                // Limit Selector
                                                <div class="w-full max-w-xs">
                                                    <EnumSelect<
                                                    FilterLimit,
                                                >
                                                        name=FilterLimitQuery::KEY
                                                        label="Limit"
                                                        value=limit
                                                        data_testid="filter-limit-select"
                                                        clear_label=FilterLimit::default().to_string()
                                                        action=InputCommitAction::SubmitForm
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
                                                        name=IncludeAdhocQuery::KEY
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
                                            <Show
                                                when=move || {
                                                    tournament_editor_map
                                                        .visible_ids_list
                                                        .with(|val| !val.is_empty())
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
                                                            each=move || {
                                                                tournament_editor_map.visible_ids_list.get()
                                                            }
                                                            key=|id| *id
                                                            children=move |id| {
                                                                view! { <TournamentTableRow id=id /> }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </Show>
                                        </div>
                                        // --- Action Bar ---
                                        <div class="flex flex-col md:flex-row justify-end gap-4">
                                            <div class:hidden=move || {
                                                tournament_editor_map.selected_id.get().is_none()
                                            }>
                                                <A
                                                    href=move || url_matched_route(
                                                        MatchedRouteHandler::Extend("edit"),
                                                    )
                                                    attr:class="btn btn-sm btn-secondary"
                                                    attr:data-testid="action-btn-edit"
                                                    scroll=false
                                                >
                                                    "Edit selected Tournament"
                                                </A>
                                            </div>
                                            <button
                                                class="btn btn-sm btn-secondary-content"
                                                class:hidden=move || {
                                                    tournament_editor_map.selected_id.get().is_none()
                                                }
                                                data-testid="action-btn-copy"
                                                on:click=move |_| {
                                                    toast_ctx.warning("Not implemented yet");
                                                }
                                            >
                                                "Copy selected Tournament"
                                            </button>
                                            <button
                                                class="btn btn-sm btn-primary"
                                                data-testid="action-btn-new"
                                                on:click=move |_| {
                                                    let navigate = use_navigate();
                                                    if let Some(new_editor) = tournament_editor_map
                                                        .spawn_editor_for_new_object(SimpleEditorOptions::no_id())
                                                        && let Some(new_id) = new_editor.base_editor.id.get()
                                                    {
                                                        let nav_url = url_matched_route_update_query(
                                                            TournamentBaseIdQuery::KEY,
                                                            &new_id.to_string(),
                                                            MatchedRouteHandler::Extend("new"),
                                                        );
                                                        navigate(
                                                            &nav_url,
                                                            NavigateOptions {
                                                                scroll: false,
                                                                ..Default::default()
                                                            },
                                                        );
                                                    } else {
                                                        toast_ctx.warning("Failed to create a new tournament");
                                                    }
                                                }
                                            >
                                                "Create new Tournament"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                                <div class="my-4"></div>
                                <Outlet />
                            }
                        })
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn TournamentTableRow(id: Uuid) -> impl IntoView {
    // --- local context ---
    let tournament_editor_map =
        expect_context::<ObjectEditorMapContext<TournamentEditorContext, TournamentBaseIdQuery>>();
    // unwrap is safe here, since we provide an id.
    let tournament_editor = tournament_editor_map
        .spawn_editor_for_edit_object(SimpleEditorOptions::with_id(id))
        .unwrap();
    let tournament_id = TournamentBaseIdQuery::use_param_query();

    // remove editor on unmount
    on_cleanup(move || {
        tournament_editor_map.remove_editor(id);
    });

    view! {
        {move || {
            tournament_editor
                .base_editor
                .load_tournament_base
                .and_then(|maybe_base| {
                    maybe_base
                        .as_ref()
                        .map(|base| {
                            tournament_editor.update_base_in_editor(base);
                            view! {
                                <tr
                                    class="hover cursor-pointer"
                                    class:bg-base-200=move || tournament_editor_map.is_selected(id)
                                    data-testid=format!("tournaments-row-{}", id)
                                    on:click=move |_| {
                                        if tournament_id.get() == Some(id) {
                                            tournament_editor_map.set_selected_id.run(None);
                                        } else {
                                            tournament_editor_map.set_selected_id.run(Some(id));
                                        }
                                    }
                                >
                                    <td
                                        class="font-bold"
                                        data-testid=format!("table-entry-name-{}", id)
                                    >
                                        {move || tournament_editor.base_editor.name.get()}
                                    </td>
                                    <td data-testid=format!("table-entry-preview-{}", id)>
                                        <p>
                                            <span class="badge badge-outline mr-2">
                                                {move || {
                                                    tournament_editor
                                                        .base_editor
                                                        .tournament_state
                                                        .get()
                                                        .map(|s| s.to_string())
                                                }}
                                            </span>
                                            {move || {
                                                tournament_editor
                                                    .base_editor
                                                    .num_entrants
                                                    .get()
                                                    .and_then(|n| {
                                                        tournament_editor
                                                            .base_editor
                                                            .mode
                                                            .get()
                                                            .map(|m| format!("{} with {} entrants", m, n))
                                                    })
                                            }}
                                        </p>
                                    </td>
                                </tr>
                                <Show when=move || tournament_editor_map.is_selected(id)>
                                    <tr>
                                        <td colspan="2" class="p-0">
                                            <div
                                                class="p-4 bg-base-100 border border-base-300 rounded-lg"
                                                data-testid="table-entry-detailed-preview"
                                            >
                                                <h3 class="font-bold text-lg mb-2">"Tournament Details"</h3>
                                                <p>
                                                    <strong>"ID: "</strong>
                                                    {move || {
                                                        tournament_editor
                                                            .base_editor
                                                            .id
                                                            .get()
                                                            .map(|id| id.to_string())
                                                    }}
                                                </p>
                                                <p>
                                                    <strong>"Type: "</strong>
                                                    {move || {
                                                        tournament_editor
                                                            .base_editor
                                                            .tournament_type
                                                            .get()
                                                            .map(|t| t.to_string())
                                                    }}
                                                </p>
                                                <p>
                                                    <strong>"State: "</strong>
                                                    {move || {
                                                        tournament_editor
                                                            .base_editor
                                                            .tournament_state
                                                            .get()
                                                            .map(|s| s.to_string())
                                                    }}
                                                </p>
                                                <p>
                                                    <strong>"Number of Entrants: "</strong>
                                                    {move || {
                                                        tournament_editor
                                                            .base_editor
                                                            .num_entrants
                                                            .get()
                                                            .map(|n| n.to_string())
                                                    }}
                                                </p>
                                            </div>
                                        </td>
                                    </tr>
                                </Show>
                            }
                        })
                })
        }}
    }
}
