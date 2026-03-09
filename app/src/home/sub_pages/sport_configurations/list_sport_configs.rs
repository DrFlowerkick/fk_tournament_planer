//! listing, creating and modifying sport configurations

use app_core::{CrTopic, SportConfig, utils::traits::ObjectIdVersion};
use app_utils::{
    components::inputs::{EnumSelect, FieldInput, InputCommitAction, InputUpdateStrategy},
    enum_utils::{EditAction, FilterLimit},
    error::{
        ComponentError,
        strategy::{handle_unexpected_ui_error, handle_with_error_banner},
    },
    hooks::{
        use_on_cancel::use_on_cancel,
        use_scroll_into_view::use_scroll_h2_into_view,
        use_url_navigation::{
            MatchedRouteHandler, UseMatchedRouteNavigationReturn, use_matched_route_navigation,
        },
    },
    params::{
        EditActionParams, FilterLimitQuery, FilterNameQuery, ParamQuery, SportConfigIdQuery,
        SportIdQuery,
    },
    server_fn::sport_config::list_sport_configs,
    state::{
        LabeledAction, SimpleEditorOptions,
        activity_tracker::ActivityTracker,
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        object_table::ObjectEditorMapContext,
        sport_config::SportConfigEditorContext,
        toast_state::ToastContext,
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use leptos::{html::H2, prelude::*};
use leptos_router::{
    NavigateOptions, components::Form, hooks::use_navigate, nested_router::Outlet,
};
use reactive_stores::Store;
use uuid::Uuid;

#[component]
pub fn ListSportConfigurations() -> impl IntoView {
    // navigation and query handling Hook
    let UseMatchedRouteNavigationReturn {
        url_is_matched_route,
        url_matched_route,
        url_matched_route_update_query,
        ..
    } = use_matched_route_navigation();

    // --- global context and state ---
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
    let sport_config_editor_map =
        ObjectEditorMapContext::<SportConfigEditorContext, SportConfigIdQuery>::new();
    provide_context(sport_config_editor_map);

    // Signals for Filters and Resource
    let sport_id = SportIdQuery::use_param_query();
    let sport_config_id = SportConfigIdQuery::use_param_query();
    let search_term = FilterNameQuery::use_param_query();
    let limit = FilterLimitQuery::use_param_query();

    // Resource that fetches data when filters change
    let sport_config_ids = Resource::new(
        move || {
            (
                sport_id.get(),
                search_term.get(),
                limit.get(),
                sport_config_editor_map.track_fetch_trigger.get(),
            )
        },
        move |(maybe_sport_id, term, lim, _)| async move {
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
                    .map_err(|app_error| ComponentError::new(component_id.get_value(), app_error))
            } else {
                Ok(vec![])
            }
        },
    );

    // Refetch callbacks
    let refetch = Callback::new(move |()| sport_config_ids.refetch());
    page_err_ctx.register_retry_handler(component_id.get_value(), refetch);

    let edit_action = EditActionParams::use_param_query();
    let reload_after_new = Callback::new(move |()| match edit_action.get_untracked() {
        Some(EditAction::New) | Some(EditAction::Copy) => {
            toast_ctx.success("New Sport Configuration on server", None);
        }
        Some(EditAction::Edit) => {
            let action = LabeledAction {
                label: "Reload List".to_string(),
                on_click: refetch,
            };

            toast_ctx.success("New Sport Configuration on server", Some(action));
        }
        None => {
            toast_ctx.success("New Sport Configuration on server, reloading list", None);
            sport_config_ids.refetch();
        }
    });

    // Subscribe to relevant events from client registry to trigger refetch
    Effect::new(move || {
        if let Some(sport_id) = sport_id.get() {
            let topic = CrTopic::NewSportConfig { sport_id };
            use_client_registry_socket(topic.into(), None.into(), reload_after_new.clone());
        }
    });

    // on_cancel handler
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <ErrorBoundary fallback=move |errors| {
            for (_err_id, err) in errors.get().into_iter() {
                let e = err.into_inner();
                if let Some(comp_err) = e.downcast_ref::<ComponentError>() {
                    handle_with_error_banner(&page_err_ctx, comp_err, on_cancel);
                } else {
                    handle_unexpected_ui_error(
                        &page_err_ctx,
                        component_id.get_value(),
                        "An unexpected error occurred.",
                        on_cancel,
                    );
                }
            }
        }>
            <Transition fallback=move || {
                view! {
                    <div
                        class="card w-full bg-base-100 shadow-xl"
                        data-testid="sport-config-list-root"
                    >
                        <div class="card-body">
                            <h2 class="card-title" node_ref=scroll_ref>
                                "Sport Configurations"
                            </h2>
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    </div>
                }
            }>
                <div class="card w-full bg-base-100 shadow-xl" data-testid="sport-config-list-root">
                    <div class="card-body">
                        <div class="flex justify-between items-center">
                            <h2 class="card-title" node_ref=scroll_ref>
                                "Sport Configurations"
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
                                    <FieldInput<
                                    String,
                                >
                                        input_type="search"
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
                        {move || {
                            sport_config_ids
                                .and_then(|sc_list| {
                                    sport_config_editor_map
                                        .visible_objects_list
                                        .set(sc_list.clone());
                                    view! {
                                        // --- Table Area ---
                                        <div class="overflow-x-auto">
                                            <Show
                                                when=move || {
                                                    sport_config_editor_map
                                                        .visible_objects_list
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
                                                            each=move || {
                                                                sport_config_editor_map
                                                                    .visible_objects_list
                                                                    .get()
                                                                    .into_iter()
                                                            }
                                                            key=|sc| sc.get_id_version()
                                                            children=move |sc| {
                                                                view! { <SportConfigTableRow sc=sc /> }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </Show>
                                        </div>
                                    }
                                })
                        }}
                        // --- Action Bar ---
                        <div class="flex flex-col md:flex-row justify-end gap-4">
                            <button
                                class="btn btn-sm btn-secondary"
                                class:hidden=move || {
                                    sport_config_editor_map.selected_id.get().is_none()
                                }
                                data-testid="action-btn-edit"
                                on:click=move |_| {
                                    let navigate = use_navigate();
                                    let nav_url = url_matched_route(
                                        MatchedRouteHandler::Extend("edit"),
                                    );
                                    navigate(
                                        &nav_url,
                                        NavigateOptions {
                                            scroll: false,
                                            ..Default::default()
                                        },
                                    );
                                }
                            >
                                "Edit selected Sport Configuration"
                            </button>
                            <button
                                class="btn btn-sm btn-secondary-content"
                                class:hidden=move || {
                                    sport_config_editor_map.selected_id.get().is_none()
                                }
                                data-testid="action-btn-copy"
                                on:click=move |_| {
                                    let navigate = use_navigate();
                                    if let Some(selected_id) = sport_config_editor_map
                                        .selected_id
                                        .get()
                                        && let Some(new_editor) = sport_config_editor_map
                                            .spawn_editor_for_copy_object(
                                                selected_id,
                                                SimpleEditorOptions::no_id(),
                                            ) && let Some(new_id) = new_editor.id.get()
                                    {
                                        let nav_url = url_matched_route_update_query(
                                            SportConfigIdQuery::KEY,
                                            &new_id.to_string(),
                                            MatchedRouteHandler::Extend("copy"),
                                        );
                                        navigate(
                                            &nav_url,
                                            NavigateOptions {
                                                scroll: false,
                                                ..Default::default()
                                            },
                                        );
                                    } else {
                                        toast_ctx
                                            .warning("Failed to copy Sport Configuration", None);
                                    }
                                }
                            >
                                "Copy selected Sport Configuration"
                            </button>
                            <button
                                class="btn btn-sm btn-primary"
                                data-testid="action-btn-new"
                                on:click=move |_| {
                                    let navigate = use_navigate();
                                    if let Some(new_editor) = sport_config_editor_map
                                        .spawn_editor_for_new_object(SimpleEditorOptions::no_id())
                                        && let Some(new_id) = new_editor.id.get()
                                    {
                                        let nav_url = url_matched_route_update_query(
                                            SportConfigIdQuery::KEY,
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
                                        toast_ctx
                                            .warning(
                                                "Failed to create a new Sport Configuration",
                                                None,
                                            );
                                    }
                                }
                            >
                                "Create new Sport Configuration"
                            </button>
                        </div>
                    </div>
                </div>
                <div class="my-4"></div>
                <Outlet />
            </Transition>
        </ErrorBoundary>
    }
}

#[component]
fn SportConfigTableRow(sc: SportConfig) -> impl IntoView {
    // sport id and plugin manager
    let sport_id = SportIdQuery::use_param_query();
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_plugin = move || {
        sport_id
            .try_get()
            .flatten()
            .and_then(|s_id| sport_plugin_manager.get().get_web_ui(&s_id))
    };

    // --- local context ---
    let sport_config_editor_map =
        expect_context::<ObjectEditorMapContext<SportConfigEditorContext, SportConfigIdQuery>>();
    let id = sc.get_id();
    // unwrap is safe here, since we provide an id.
    let sport_config_editor = sport_config_editor_map
        .spawn_editor_for_edit_object(SimpleEditorOptions::with_id(id))
        .unwrap();
    // We use update here, because there is a chain of events, which might result in an unwanted edit conflict. If
    // the user changes several fields, the entry will differ with regard to the last loaded list entry. If during
    // editing a reload of the list is triggered (e.g. because of adding a new entry to the list), there is a tiny
    // window, where the user changes a field and triggers a save before the list is fully loaded. This results
    // in an edit conflict, because the save tries to update an outdated version of the entry. By using update here,
    // we only update the editor with the list data, if it is newer than the optimistic version of the edited entry.
    sport_config_editor_map.update_object_in_editor(&sc);
    let sport_config_id = SportConfigIdQuery::use_param_query();

    view! {
        {move || {
            sport_plugin()
                .map(|sp| {
                    let sp = StoredValue::new(sp);
                    view! {
                        <tr
                            class="hover cursor-pointer"
                            class:bg-base-200=move || { sport_config_editor_map.is_selected(id) }
                            data-testid=format!("table-entry-row-{}", id)
                            on:click=move |_| {
                                if sport_config_id.get() == Some(id) {
                                    sport_config_editor_map.set_selected_id.run(None);
                                } else {
                                    sport_config_editor_map.set_selected_id.run(Some(id));
                                }
                            }
                        >
                            <td class="font-bold" data-testid=format!("table-entry-name-{}", id)>
                                {move || sport_config_editor.name.get()}
                            </td>
                            <td data-testid=format!(
                                "table-entry-preview-{}",
                                id,
                            )>
                                {move || {
                                    sport_config_editor
                                        .local_read_only
                                        .with(|local| {
                                            local
                                                .as_ref()
                                                .map(|sc| { sp.get_value().render_preview(sc) })
                                        })
                                }}
                            </td>
                        </tr>
                        <Show when=move || sport_config_editor_map.is_selected(id)>
                            <tr>
                                <td colspan="2" class="p-0">
                                    {move || {
                                        sport_config_editor
                                            .local_read_only
                                            .with(|local| {
                                                local
                                                    .as_ref()
                                                    .map(|sc| { sp.get_value().render_detailed_preview(sc) })
                                            })
                                    }}
                                </td>
                            </tr>
                        </Show>
                    }
                })
        }}
    }
}
