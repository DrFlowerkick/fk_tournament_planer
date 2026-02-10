//! Postal Address Search Component

use app_core::CrTopic;
use app_utils::{
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
    server_fn::postal_address::list_postal_addresses,
    state::{
        activity_tracker::ActivityTracker, error_state::PageErrorContext,
        postal_address::PostalAddressListContext,
    },
};
use cr_leptos_axum_socket::use_client_registry_socket;
//use cr_single_instance::use_client_registry_sse;
use isocountry::CountryCode;
use leptos::{html::H2, prelude::*};
use leptos_router::{NavigateOptions, components::A, hooks::use_navigate, nested_router::Outlet};
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
        url_update_query,
        url_remove_query,
        url_matched_route,
        url_is_matched_route,
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    // --- global context and state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();

    // remove errors and activity tracker on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // --- local context ---
    let postal_address_list_ctx = PostalAddressListContext::new();
    provide_context(postal_address_list_ctx);

    // Signals for Filters
    // ToDo: consider using query search params as described in
    // https://book.leptos.dev/router/20_form.html
    // This would allow users to share filtered views via URL and preserve filter state on page reloads.
    let (search_term, set_search_term) = signal("".to_string());
    let (limit, set_limit) = signal(10usize);
    // Signal for Selected Row (UI interaction)
    let (selected_id, set_selected_id) = signal::<Option<Uuid>>(None);

    // update address_id query param when selected_id changes
    let handle_selection_change = Callback::new({
        let navigate = navigate.clone();
        move |new_id: Option<Uuid>| {
            set_selected_id.set(new_id);

            let nav_url = if let Some(t_id) = new_id {
                url_update_query("address_id", &t_id.to_string())
            } else {
                url_remove_query("address_id")
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

    // Resource that fetches data when filters change
    let postal_addresses_data = Resource::new(
        move || {
            (
                search_term.get(),
                limit.get(),
                postal_address_list_ctx.track_fetch_trigger.get(),
            )
        },
        move |(term, lim, _refetch_trigger)| async move {
            activity_tracker
                .track_activity_wrapper(
                    component_id.get_value(),
                    list_postal_addresses(term, Some(lim)),
                )
                .await
        },
    );

    // Refetch function for errors
    let refetch = Callback::new(move |()| postal_addresses_data.refetch());

    // on_cancel handler
    let on_cancel = use_on_cancel();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <div class="card w-full bg-base-100 shadow-xl" data-testid="postal-address-list-root">
            <div class="card-body">
                <h2 class="card-title" node_ref=scroll_ref>
                    "Search Postal Address"
                </h2>
                // --- Action Bar ---
                <div class="flex flex-col md:flex-row justify-end gap-4">
                    <A
                        href=move || url_matched_route_remove_query(
                            "address_id",
                            MatchedRouteHandler::Extend("new"),
                        )
                        attr:class="btn btn-sm btn-primary"
                        attr:data-testid="action-btn-new"
                        scroll=false
                    >
                        "Create New Postal Address"
                    </A>
                </div>

                // --- Filter Bar ---
                <div class="bg-base-200 p-4 rounded-lg flex flex-wrap gap-4 items-end">
                    // Text Search
                    <div class="form-control w-full max-w-xs">
                        <label class="label">
                            <span class="label-text">"Search Name"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="Type to search for name..."
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
                                postal_addresses_data
                                    .and_then(|data| {
                                        if let Some(selected_id) = selected_id.get_untracked()
                                            && !data.iter().any(|t| t.get_id() == selected_id)
                                        {
                                            handle_selection_change.run(None);
                                        }
                                        let data = StoredValue::new(data.clone());
                                        view! {
                                            <Show
                                                when=move || data.with_value(|val| !val.is_empty())
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
                                                <table
                                                    class="table w-full"
                                                    data-testid="postal-address-table"
                                                >
                                                    <thead data-testid="postal-address-table-header">
                                                        <tr>
                                                            <th>"Name"</th>
                                                            <th>"Preview"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <For
                                                            each=move || data.read_value().clone()
                                                            key=|pa| pa.get_id()
                                                            children=move |pa| {
                                                                let pa_id = pa.get_id();
                                                                let is_selected = move || {
                                                                    selected_id.get() == Some(pa_id)
                                                                };
                                                                let topic = Signal::derive(move || {
                                                                    Some(CrTopic::Address(pa_id))
                                                                });
                                                                let version = Signal::derive({
                                                                    let pa = pa.clone();
                                                                    move || { pa.get_version().unwrap_or_default() }
                                                                });
                                                                use_client_registry_socket(topic, version, refetch);
                                                                view! {
                                                                    <tr
                                                                        class="hover cursor-pointer"
                                                                        class:bg-base-200=is_selected
                                                                        data-testid=format!("postal-address-row-{}", pa_id)
                                                                        on:click=move |_| {
                                                                            if selected_id.get() == Some(pa_id) {
                                                                                handle_selection_change.run(None);
                                                                            } else {
                                                                                handle_selection_change.run(Some(pa_id));
                                                                            }
                                                                        }
                                                                    >
                                                                        <td
                                                                            class="font-bold"
                                                                            data-testid=format!("postal-address-name-{}", pa_id)
                                                                        >
                                                                            {pa.get_name().to_string()}
                                                                        </td>
                                                                        <td data-testid=format!("postal-address-preview-{}", pa_id)>
                                                                            <div
                                                                                class="card w-full bg-base-200 shadow-md mt-4"
                                                                                data-testid="address-preview"
                                                                            >
                                                                                <div class="card-body">
                                                                                    <h3 class="card-title" data-testid="preview-address-name">
                                                                                        {pa.get_name().to_string()}
                                                                                    </h3>
                                                                                    <p data-testid="preview-street">
                                                                                        {pa.get_street().to_string()}
                                                                                    </p>
                                                                                    <p data-testid="preview-postal_locality">
                                                                                        <span data-testid="preview-postal_code">
                                                                                            {pa.get_postal_code().to_string()}
                                                                                        </span>
                                                                                        " "
                                                                                        <span data-testid="preview-locality">
                                                                                            {pa.get_locality().to_string()}
                                                                                        </span>
                                                                                    </p>
                                                                                    <p data-testid="preview-region">
                                                                                        {pa.get_region().unwrap_or_default().to_string()}
                                                                                    </p>
                                                                                    <p data-testid="preview-country">
                                                                                        {display_country(pa.get_country())}
                                                                                    </p>
                                                                                    <p class="hidden" data-testid="preview-address-id">
                                                                                        {pa.get_id().to_string()}
                                                                                    </p>
                                                                                    <p class="hidden" data-testid="preview-address-version">
                                                                                        {pa.get_version().unwrap_or_default()}
                                                                                    </p>
                                                                                </div>
                                                                            </div>
                                                                        </td>
                                                                    </tr>
                                                                    <Show when=is_selected>
                                                                        <tr>
                                                                            <td colspan="4" class="p-0">
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
