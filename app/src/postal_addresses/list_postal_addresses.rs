//! Postal Address Search Component

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
    params::{AddressIdQuery, FilterLimitQuery, FilterNameQuery, ParamQuery},
    server_fn::postal_address::{list_postal_address_ids, load_postal_address},
    state::{
        EditorContext, activity_tracker::ActivityTracker, error_state::PageErrorContext,
        object_table::ObjectEditorMapContext, postal_address::PostalAddressEditorContext,
        toast_state::ToastContext,
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
use isocountry::CountryCode;
use leptos::{html::H2, prelude::*};
use leptos_router::{
    NavigateOptions,
    components::{A, Form},
    hooks::use_navigate,
    nested_router::Outlet,
};
use uuid::Uuid;

fn display_country(country_code: Option<CountryCode>) -> String {
    country_code
        .map(|c| format!("{} ({})", c.name(), c.alpha2()))
        .unwrap_or_default()
}

#[component]
pub fn ListPostalAddresses() -> impl IntoView {
    // navigation helpers
    let UseQueryNavigationReturn {
        url_is_matched_route,
        url_matched_route,
        url_matched_route_update_query,
        ..
    } = use_query_navigation();

    // --- global context and state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let toast_ctx = expect_context::<ToastContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();

    // remove errors and activity tracker on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- local context ---
    let postal_address_editor_map =
        ObjectEditorMapContext::<PostalAddressEditorContext, AddressIdQuery>::new();
    provide_context(postal_address_editor_map);

    // Signals for Filters
    let address_id = AddressIdQuery::use_param_query();
    let search_term = FilterNameQuery::use_param_query();
    let limit = FilterLimitQuery::use_param_query();

    // Resource that fetches data when filters change
    let postal_address_ids = Resource::new(
        move || {
            (
                search_term.get(),
                limit.get(),
                postal_address_editor_map.track_fetch_trigger.get(),
            )
        },
        move |(term, lim, _)| async move {
            activity_tracker
                .track_activity_wrapper(
                    component_id.get_value(),
                    list_postal_address_ids(
                        term.unwrap_or_default(),
                        lim.or_else(|| Some(FilterLimit::default()))
                            .map(|l| l as usize),
                    ),
                )
                .await
        },
    );

    // Refetch function for errors
    let refetch = Callback::new(move |()| postal_address_ids.refetch());

    // on_cancel handler
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <Transition fallback=move || {
            view! {
                <div
                    class="card w-full bg-base-100 shadow-xl"
                    data-testid="postal-address-list-root"
                >
                    <div class="card-body">
                        <h2 class="card-title" node_ref=scroll_ref>
                            "Search Postal Address"
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
                    postal_address_ids
                        .and_then(|pa_ids| {
                            let pa_ids = StoredValue::new(pa_ids.clone());
                            view! {
                                <div
                                    class="card w-full bg-base-100 shadow-xl"
                                    data-testid="postal-address-list-root"
                                >
                                    <div class="card-body">
                                        <h2 class="card-title" node_ref=scroll_ref>
                                            "Search Postal Address"
                                        </h2>

                                        // --- Filter Bar ---
                                        <Form method="GET" action="" noscroll=true replace=true>
                                            // Hidden input to keep address_id in query string
                                            <input
                                                type="hidden"
                                                name=AddressIdQuery::KEY
                                                prop:value=move || {
                                                    address_id
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
                                                when=move || { pa_ids.with_value(|val| !val.is_empty()) }
                                                fallback=|| {
                                                    view! {
                                                        <div
                                                            class="text-center py-10 bg-base-100 border border-base-300 rounded-lg"
                                                            data-testid="postal-address-list-empty"
                                                        >
                                                            <p class="text-lg opacity-60">
                                                                "No postal addresses found with the current filters."
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
                                                            each=move || { pa_ids.get_value().into_iter() }
                                                            key=|id| *id
                                                            children=move |id| {
                                                                view! { <PostalAddressTableRow id=id /> }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </Show>
                                        </div>
                                        // --- Action Bar ---
                                        <div class="flex flex-col md:flex-row justify-end gap-4">
                                            <div class:hidden=move || {
                                                postal_address_editor_map.selected_id.get().is_none()
                                            }>
                                                <A
                                                    href=move || url_matched_route(
                                                        MatchedRouteHandler::Extend("edit"),
                                                    )
                                                    attr:class="btn btn-sm btn-secondary"
                                                    attr:data-testid="action-btn-edit"
                                                    scroll=false
                                                >
                                                    "Edit selected Postal Address"
                                                </A>
                                            </div>
                                            <button
                                                class="btn btn-sm btn-secondary-content"
                                                class:hidden=move || {
                                                    postal_address_editor_map.selected_id.get().is_none()
                                                }
                                                data-testid="action-btn-copy"
                                                on:click=move |_| {
                                                    let navigate = use_navigate();
                                                    if let Some(new_id) = postal_address_editor_map
                                                        .copy_editor
                                                        .run(())
                                                    {
                                                        let nav_url = url_matched_route_update_query(
                                                            AddressIdQuery::KEY,
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
                                                        toast_ctx.warning("Failed to copy object");
                                                    }
                                                }
                                            >
                                                "Copy selected Postal Address"
                                            </button>
                                            <button
                                                class="btn btn-sm btn-primary"
                                                data-testid="action-btn-new"
                                                on:click=move |_| {
                                                    let navigate = use_navigate();
                                                    if let Some(new_id) = postal_address_editor_map
                                                        .new_editor
                                                        .run(())
                                                    {
                                                        let nav_url = url_matched_route_update_query(
                                                            AddressIdQuery::KEY,
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
                                                        toast_ctx.warning("Failed to create a new postal address");
                                                    }
                                                }
                                            >
                                                "Create new Postal Address"
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
fn PostalAddressTableRow(#[prop(into)] id: Signal<Uuid>) -> impl IntoView {
    // --- local context ---
    let postal_address_editor_map =
        expect_context::<ObjectEditorMapContext<PostalAddressEditorContext, AddressIdQuery>>();
    let postal_address_editor = PostalAddressEditorContext::new();
    postal_address_editor_map.insert_editor(id.get(), postal_address_editor);
    let address_id = AddressIdQuery::use_param_query();

    // --- global state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let activity_tracker = expect_context::<ActivityTracker>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // resource to load postal address
    // since we render PostalAddressTableRow inside the Transition block of ListPostalAddresses,
    // we do not need to use another Transition block to load the postal address.
    /*let list_entry_addr_res = Resource::new(
        move || id.get(),
        move |id| async move {
            match activity_tracker
                .track_activity_wrapper(component_id.get_value(), load_postal_address(id))
                .await
            {
                Ok(Some(pa)) => Ok(pa),
                Ok(None) => Err(AppError::ResourceNotFound("Postal Address".to_string(), id)),
                Err(err) => Err(err),
            }
        },
    );*/
    // At current state of leptos SSR does not provide stable rendering (meaning during initial load Hydration
    // errors occur until the page is fully rendered and the app "transformed" into a SPA). For this reason
    // we use a LocalResource here, which does not cause hydration errors.
    // ToDo: investigate how to use Resource without hydration errors, since Resource provides better
    // ergonomics for loading states and error handling.
    let list_entry_addr_res = LocalResource::new(move || async move {
        match activity_tracker
            .track_activity_wrapper(component_id.get_value(), load_postal_address(id.get()))
            .await
        {
            Ok(Some(pa)) => Ok(pa),
            Ok(None) => Err(AppError::ResourceNotFound(
                "Postal Address".to_string(),
                id.get(),
            )),
            Err(err) => Err(err),
        }
    });

    let topic = Signal::derive(move || Some(CrTopic::Address(id.get())));
    let refetch = Callback::new(move |()| {
        list_entry_addr_res.refetch();
    });
    use_client_registry_socket(topic, postal_address_editor.optimistic_version, refetch);

    view! {
        {move || {
            list_entry_addr_res
                .and_then(|pa| {
                    postal_address_editor_map.update_object_in_editor(pa);
                    view! {
                        <tr
                            class="hover cursor-pointer"
                            class:bg-base-200=move || {
                                postal_address_editor_map.is_selected(id.get())
                            }
                            data-testid=format!("table-entry-row-{}", id.get())
                            on:click=move |_| {
                                if address_id.get() == Some(id.get()) {
                                    postal_address_editor_map.set_selected_id.run(None);
                                } else {
                                    postal_address_editor_map.set_selected_id.run(Some(id.get()));
                                }
                            }
                        >
                            <td
                                class="font-bold"
                                data-testid=format!("table-entry-name-{}", id.get())
                            >
                                {move || postal_address_editor.name.get()}
                            </td>
                            <td data-testid=format!(
                                "table-entry-preview-{}",
                                id.get(),
                            )>
                                {move || {
                                    format!(
                                        "{} - {}",
                                        postal_address_editor.locality.get().unwrap_or_default(),
                                        postal_address_editor
                                            .country
                                            .get()
                                            .map(|c| c.name())
                                            .unwrap_or_default(),
                                    )
                                }}
                            </td>
                        </tr>
                        <Show when=move || postal_address_editor_map.is_selected(id.get())>
                            <tr>
                                <td colspan="2" class="p-0">
                                    <div
                                        class="flex flex-wrap items-baseline gap-x-2 gap-y-1 p-4 bg-base-200 text-sm"
                                        data-testid="table-entry-detailed-preview"
                                    >
                                        <span data-testid="preview-street" class="font-medium">
                                            {move || postal_address_editor.street.get()}
                                        </span>

                                        <span class="opacity-50 hidden sm:inline">"•"</span>

                                        <span data-testid="preview-postal_locality">
                                            <span data-testid="preview-postal_code">
                                                {move || postal_address_editor.postal_code.get()}
                                            </span>
                                            " "
                                            <span data-testid="preview-locality">
                                                {move || postal_address_editor.locality.get()}
                                            </span>
                                        </span>

                                        <Show when=move || {
                                            postal_address_editor.region.get().is_some()
                                        }>
                                            <span class="opacity-50 hidden sm:inline">"•"</span>
                                            <span data-testid="preview-region">
                                                {move || {
                                                    postal_address_editor
                                                        .region
                                                        .get()
                                                        .map(|r| r.to_string())
                                                        .unwrap_or_default()
                                                }}
                                            </span>
                                        </Show>

                                        <span class="opacity-50 hidden sm:inline">"•"</span>

                                        <span
                                            data-testid="preview-country"
                                            class="text-base-content/70"
                                        >
                                            {move || display_country(
                                                postal_address_editor.country.get(),
                                            )}
                                        </span>

                                        // Hidden technical fields
                                        <span class="hidden" data-testid="preview-address-id">
                                            {move || {
                                                postal_address_editor
                                                    .id
                                                    .get()
                                                    .map(|id| id.to_string())
                                                    .unwrap_or_default()
                                            }}
                                        </span>
                                        <span class="hidden" data-testid="preview-address-version">
                                            {move || {
                                                postal_address_editor.version.get().unwrap_or_default()
                                            }}
                                        </span>
                                    </div>
                                </td>
                            </tr>
                            <tr>
                                <td colspan="2" class="p-0">
                                    <div
                                        class="flex gap-2 justify-end p-2 bg-base-200"
                                        data-testid="row-actions"
                                    ></div>
                                </td>
                            </tr>
                        </Show>
                    }
                })
        }}
    }
}
