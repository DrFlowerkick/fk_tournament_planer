//! listing, creating and modifying sport configurations

use app_core::CrTopic;
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
    params::{FilterLimitQuery, FilterNameQuery, ParamQuery, SportConfigIdQuery, SportIdQuery},
    server_fn::sport_config::{list_sport_config_ids, load_sport_config},
    state::{
        EditorContext,
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        sport_config::SportConfigEditorContext,
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::{html::H2, prelude::*};
use leptos_router::{
    NavigateOptions,
    components::{A, Form},
    hooks::use_navigate,
    nested_router::Outlet,
};
use reactive_stores::Store;
use uuid::Uuid;

#[component]
pub fn ListSportConfigurations() -> impl IntoView {
    // navigation and query handling Hook
    let UseQueryNavigationReturn {
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

    // --- local context ---
    let sport_config_editor = SportConfigEditorContext::new();
    provide_context(sport_config_editor);
    let origin = move || {
        if sport_config_editor.has_origin().get() {
            "Has Origin".to_string()
        } else {
            "No Origin".to_string()
        }
    };
    let direct_origin = move || {
        if sport_config_editor.origin.get().is_some() {
            "Has direct Origin".to_string()
        } else {
            "No direct Origin".to_string()
        }
    };

    // Signals for Filters and Resource
    let sport_id = SportIdQuery::use_param_query();
    let sport_config_id = SportConfigIdQuery::use_param_query();
    let search_term = FilterNameQuery::use_param_query();
    let limit = FilterLimitQuery::use_param_query();

    // Resource that fetches data when filters change
    let sport_config_ids = Resource::new(
        move || (sport_id.get(), search_term.get(), limit.get()),
        move |(maybe_sport_id, term, lim)| async move {
            if let Some(s_id) = maybe_sport_id {
                activity_tracker
                    .track_activity_wrapper(
                        component_id.get_value(),
                        list_sport_config_ids(
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
    let refetch = Callback::new(move |()| sport_config_ids.refetch());

    // on_cancel handler
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
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
                    sport_config_ids
                        .and_then(|sc_ids| {
                            let sc_ids = StoredValue::new(sc_ids.clone());

                            view! {
                                <div
                                    class="card w-full bg-base-100 shadow-xl"
                                    data-testid="sport-config-list-root"
                                >
                                    <div class="card-body">
                                        <h2 class="card-title" node_ref=scroll_ref>
                                            "Sport Configurations"
                                        </h2>
                                        <p class="text-primary">{move || origin()}</p>
                                        <p class="text-secondary">{move || direct_origin()}</p>

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
                                                name=SportIdQuery::KEY
                                                prop:value=move || {
                                                    sport_id.get().map(|id| id.to_string()).unwrap_or_default()
                                                }
                                            />
                                            <input
                                                type="hidden"
                                                name=SportConfigIdQuery::KEY
                                                prop:value=move || {
                                                    sport_config_id
                                                        .get()
                                                        .map(|id| id.to_string())
                                                        .unwrap_or_default()
                                                }
                                            />
                                            <div class="bg-base-200 p-4 rounded-lg flex flex-wrap gap-4 items-end">
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
                                            </div>
                                        </Form>

                                        // --- Table Area ---
                                        <div class="overflow-x-auto">
                                            <Show
                                                when=move || { sc_ids.with_value(|val| !val.is_empty()) }
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
                                                            each=move || { sc_ids.get_value().into_iter() }
                                                            key=|id| *id
                                                            children=move |id| {
                                                                view! { <SportConfigTableRow id=id /> }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </Show>

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
fn SportConfigTableRow(#[prop(into)] id: Signal<Uuid>) -> impl IntoView {
    // navigation helpers
    let UseQueryNavigationReturn {
        url_update_query,
        url_remove_query,
        url_matched_route,
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();
    let sport_config_id = SportConfigIdQuery::use_param_query();
    let is_selected = Memo::new(move |_| sport_config_id.get() == Some(id.get()));

    // sport id and plugin manager
    let sport_id = SportIdQuery::use_param_query();
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_plugin = move || {
        sport_id
            .get()
            .and_then(|id| sport_plugin_manager.get().get_web_ui(&id))
    };

    // --- local context ---
    let sport_config_editor = expect_context::<SportConfigEditorContext>();

    // Callback for updating the selected sport config id, which updates the query string and thus the URL
    let set_selected_id = Callback::new(move |selected_id: Option<Uuid>| {
        let nav_url = if let Some(id) = selected_id {
            url_update_query(SportConfigIdQuery::KEY, &id.to_string())
        } else {
            url_remove_query(SportConfigIdQuery::KEY)
        };
        navigate(
            &nav_url,
            NavigateOptions {
                replace: true,
                scroll: false,
                ..Default::default()
            },
        );
    });

    // --- global state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let activity_tracker = expect_context::<ActivityTracker>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // resource to load sport config
    // since we render SportConfigTableRow inside the Transition block of ListSportConfigs,
    // we do not need to use another Transition block to load the sport config.
    /*let list_entry_sport_config_res = Resource::new(
        move || id.get(),
        move |id| async move {
            match activity_tracker
                .track_activity_wrapper(component_id.get_value(), load_sport_config(id))
                .await
            {
                Ok(Some(sc)) => Ok(sc),
                Ok(None) => Err(AppError::ResourceNotFound("Sport Config".to_string(), id)),
                Err(err) => Err(err),
            }
        },
    );*/
    // At current state of leptos SSR does not provide stable rendering (meaning during initial load Hydration
    // errors occur until the page is fully rendered and the app "transformed" into a SPA). For this reason
    // we use a LocalResource here, which does not cause hydration errors.
    // ToDo: investigate how to use Resource without hydration errors, since Resource provides better
    // ergonomics for loading states and error handling.
    let list_entry_sport_config_res = LocalResource::new(move || async move {
        match activity_tracker
            .track_activity_wrapper(component_id.get_value(), load_sport_config(id.get()))
            .await
        {
            Ok(Some(sc)) => Ok(sc),
            Ok(None) => Err(AppError::ResourceNotFound(
                "Sport Config".to_string(),
                id.get(),
            )),
            Err(err) => Err(err),
        }
    });

    let topic = Signal::derive(move || Some(CrTopic::SportConfig(id.get())));
    let version = RwSignal::new(None::<u32>);
    let refetch = Callback::new(move |()| {
        list_entry_sport_config_res.refetch();
    });
    use_client_registry_socket(topic, version.into(), refetch);

    view! {
        {move || {
            list_entry_sport_config_res
                .and_then(|sc| {
                    version.set(sc.get_version());
                    let sc = RwSignal::new(sc.clone());
                    /*Effect::new(move || {
                        if is_selected.get() {
                            leptos::logging::log!("Setting Object {} with id {} in editor context", sc.read().get_name(), sc.read().get_id());
                        } else {
                            leptos::logging::log!("Object {} with id {} is not selected, skipping setting editor context", sc.read().get_name(), sc.read().get_id());
                        }
                    });*/
                    sport_plugin()
                        .map(|sp| {
                            let sp = StoredValue::new(sp);
                            view! {
                                <tr
                                    class="hover cursor-pointer"
                                    class:bg-base-200=move || is_selected.get()
                                    data-testid=format!("table-entry-row-{}", sc.read().get_id())
                                    on:click=move |_| {
                                        if sport_config_id.get() == Some(sc.read().get_id()) {
                                            set_selected_id.run(None);
                                            sport_config_editor.set_version_signal(None);
                                        } else {
                                            set_selected_id.run(Some(sc.read().get_id()));
                                            sport_config_editor.set_version_signal(Some(version));
                                            sport_config_editor.set_sport_config(sc.get());
                                        }
                                    }
                                >
                                    <td
                                        class="font-bold"
                                        data-testid=format!(
                                            "table-entry-name-{}",
                                            sc.read().get_id(),
                                        )
                                    >
                                        {sc.read().get_name().to_string()}
                                    </td>
                                    <td data-testid=format!(
                                        "table-entry-preview-{}",
                                        sc.read().get_id(),
                                    )>
                                        {move || {
                                            sp.get_value().render_preview(sc.read().as_borrowed())
                                        }}
                                    </td>
                                </tr>
                                <Show when=move || is_selected.get()>
                                    <tr>
                                        <td colspan="2" class="p-0">
                                            {move || {
                                                sp.get_value()
                                                    .render_detailed_preview(sc.read().as_borrowed())
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
                                                    href=move || url_matched_route_remove_query(
                                                        SportConfigIdQuery::KEY,
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
                        })
                })
        }}
    }
}
