//! listing, creating and modifying sport configurations

use app_core::{CrTopic, SportConfig};
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
    params::{FilterLimitQuery, FilterNameQuery, ParamQuery, SportConfigIdQuery, SportIdQuery},
    server_fn::sport_config::list_sport_configs,
    state::{
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        object_table_list::ObjectListContext,
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::{html::H2, prelude::*};
use leptos_router::{
    components::{A, Form},
    nested_router::Outlet,
};
use reactive_stores::Store;
use uuid::Uuid;

#[component]
pub fn ListSportConfigurations() -> impl IntoView {
    // navigation and query handling Hook
    let UseQueryNavigationReturn {
        url_matched_route,
        url_is_matched_route,
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();

    // --- global context and state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });
    // Derived Query Params
    let sport_id = SportIdQuery::use_param_query();
    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_plugin = move || {
        sport_id
            .get()
            .and_then(|id| sport_plugin_manager.get().get_web_ui(&id))
    };

    // --- local context ---
    let sport_config_list_ctx = ObjectListContext::<SportConfig, SportConfigIdQuery>::new();
    provide_context(sport_config_list_ctx);

    // Signals for Filters
    let sport_config_id = SportConfigIdQuery::use_param_query();
    let search_term = FilterNameQuery::use_param_query();
    let limit = FilterLimitQuery::use_param_query();

    // Resource that fetches data when filters change
    let sport_configs_data = Resource::new(
        move || {
            (
                sport_id.get(),
                search_term.get(),
                limit.get(),
                sport_config_list_ctx.track_fetch_trigger.get(),
            )
        },
        move |(maybe_sport_id, term, lim, _refetch_trigger)| async move {
            if let Some(s_id) = maybe_sport_id {
                activity_tracker
                    .track_activity_wrapper(
                        component_id.get_value(),
                        list_sport_configs(
                            s_id,
                            term.unwrap_or_default(),
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
    let refetch = Callback::new(move |()| sport_configs_data.refetch());

    // on_cancel handler
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <div class="card w-full bg-base-100 shadow-xl" data-testid="sport-config-list-root">
            <div class="card-body">
                <h2 class="card-title" node_ref=scroll_ref>
                    "Sport Configurations"
                </h2>

                // --- Action Bar ---
                <div class="flex flex-col md:flex-row justify-end gap-4">
                    <A
                        href=move || url_matched_route_remove_query(
                            "sport_config_id",
                            MatchedRouteHandler::Extend("new"),
                        )
                        attr:class="btn btn-sm btn-primary"
                        attr:data-testid="action-btn-new"
                        scroll=false
                    >
                        "Create New Configuration"
                    </A>
                </div>

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
                        name=SportConfigIdQuery::key()
                        prop:value=move || {
                            sport_config_id.get().map(|id| id.to_string()).unwrap_or_default()
                        }
                    />
                    <div class="bg-base-200 p-4 rounded-lg flex flex-wrap gap-4 items-end">
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
                                sport_configs_data
                                    .and_then(|data| {
                                        sport_config_list_ctx.object_list.set(data.clone());
                                        sport_plugin()
                                            .map(|sp| {
                                                let sp = StoredValue::new(sp);
                                                view! {
                                                    <Show
                                                        when=move || {
                                                            sport_config_list_ctx
                                                                .object_list
                                                                .with(|val| !val.is_empty())
                                                        }
                                                        fallback=|| {
                                                            view! {
                                                                <div
                                                                    class="text-center py-10 bg-base-100 border border-base-300 rounded-lg"
                                                                    data-testid="sport-configs-list-empty"
                                                                >
                                                                    <p class="text-lg opacity-60">
                                                                        "No sport configurations found with the current filters."
                                                                    </p>
                                                                </div>
                                                            }
                                                        }
                                                    >
                                                        <table class="table w-full" data-testid="table-list">
                                                            <thead data-testid="table-list-header">
                                                                <tr>
                                                                    <th>"Name"</th>
                                                                    <th>"Preview"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                <For
                                                                    each=move || { sport_config_list_ctx.object_list.get() }
                                                                    key=|sc| sc.get_id()
                                                                    children=move |sc| {
                                                                        let sc = StoredValue::new(sc);
                                                                        let is_selected = move || {
                                                                            sport_config_list_ctx.selected_id.get()
                                                                                == Some(sc.read_value().get_id())
                                                                        };
                                                                        let topic = Signal::derive(move || {
                                                                            Some(CrTopic::SportConfig(sc.read_value().get_id()))
                                                                        });
                                                                        let version = Signal::derive({
                                                                            move || {
                                                                                sc.read_value().get_version().unwrap_or_default()
                                                                            }
                                                                        });
                                                                        use_client_registry_socket(topic, version, refetch);
                                                                        view! {
                                                                            <tr
                                                                                class="hover cursor-pointer"
                                                                                class:bg-base-200=is_selected
                                                                                data-testid=format!(
                                                                                    "table-entry-row-{}",
                                                                                    sc.read_value().get_id(),
                                                                                )
                                                                                on:click=move |_| {
                                                                                    if sport_config_list_ctx.selected_id.get()
                                                                                        == Some(sc.read_value().get_id())
                                                                                    {
                                                                                        sport_config_list_ctx.set_selected_id.run(None);
                                                                                    } else {
                                                                                        sport_config_list_ctx
                                                                                            .set_selected_id
                                                                                            .run(Some(sc.read_value().get_id()));
                                                                                    }
                                                                                }
                                                                            >
                                                                                <td
                                                                                    class="font-bold"
                                                                                    data-testid=format!(
                                                                                        "table-entry-name-{}",
                                                                                        sc.read_value().get_id(),
                                                                                    )
                                                                                >
                                                                                    {sc.read_value().get_name().to_string()}
                                                                                </td>
                                                                                <td data-testid=format!(
                                                                                    "table-entry-preview-{}",
                                                                                    sc.read_value().get_id(),
                                                                                )>
                                                                                    {move || {
                                                                                        sp.get_value().render_preview(sc.read_value().as_borrowed())
                                                                                    }}
                                                                                </td>
                                                                            </tr>
                                                                            <Show when=is_selected>
                                                                                <tr>
                                                                                    <td colspan="2" class="p-0">
                                                                                        {move || {
                                                                                            sp.get_value()
                                                                                                .render_detailed_preview(sc.read_value().as_borrowed())
                                                                                        }}
                                                                                    </td>
                                                                                </tr>
                                                                                <tr>
                                                                                    <td colspan="2" class="p-0">
                                                                                        <div
                                                                                            class="flex gap-2 justify-end p-2 bg-base-200"
                                                                                            data-testid="row-actions"
                                                                                        >
                                                                                            <A
                                                                                                href=move || url_matched_route(
                                                                                                    MatchedRouteHandler::Extend("edit"),
                                                                                                )
                                                                                                attr:class="btn btn-sm btn-primary"
                                                                                                attr:data-testid="action-btn-edit"
                                                                                                scroll=false
                                                                                            >
                                                                                                "Edit"
                                                                                            </A>
                                                                                            <A
                                                                                                href=move || url_matched_route(
                                                                                                    MatchedRouteHandler::Extend("copy"),
                                                                                                )
                                                                                                attr:class="btn btn-sm btn-ghost"
                                                                                                attr:data-testid="action-btn-copy"
                                                                                                scroll=false
                                                                                            >
                                                                                                "Copy"
                                                                                            </A>
                                                                                        </div>
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
