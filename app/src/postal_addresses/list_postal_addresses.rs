//! Postal Address Search Component

use app_core::{CrTopic, PostalAddress};
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
    state::{activity_tracker::ActivityTracker, error_state::PageErrorContext},
};
use cr_leptos_axum_socket::use_client_registry_socket;
//use cr_single_instance::use_client_registry_sse;
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
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();

    // --- global context and state ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    let activity_tracker = expect_context::<ActivityTracker>();

    // remove errors and activity tracker on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
        activity_tracker.remove_component(component_id.get_value());
    });

    // Signals for Filters
    let address_id = AddressIdQuery::use_param_query();
    let search_term = FilterNameQuery::use_param_query();
    let limit = FilterLimitQuery::use_param_query();

    // Resource that fetches data when filters change
    let postal_address_ids = Resource::new(
        move || (search_term.get(), limit.get()),
        move |(term, lim)| async move {
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
        <div class="card w-full bg-base-100 shadow-xl" data-testid="postal-address-list-root">
            <div class="card-body">
                <h2 class="card-title" node_ref=scroll_ref>
                    "Search Postal Address"
                </h2>
                // --- Action Bar ---
                <div class="flex flex-col md:flex-row justify-end gap-4">
                    <A
                        href=move || url_matched_route_remove_query(
                            AddressIdQuery::KEY,
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
                <Form method="GET" action="" noscroll=true replace=true>
                    // Hidden input to keep address_id in query string
                    <input
                        type="hidden"
                        name=AddressIdQuery::KEY
                        prop:value=move || {
                            address_id.get().map(|id| id.to_string()).unwrap_or_default()
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
                                postal_address_ids
                                    .and_then(|pa_ids| {
                                        let pa_ids = StoredValue::new(pa_ids.clone());
                                        view! {
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
                                                            each=move || { pa_ids.get_value() }
                                                            key=|id| *id
                                                            children=move |id| {
                                                                view! { <LoadPostalAddressTableRow id=id /> }
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
fn LoadPostalAddressTableRow(id: Uuid) -> impl IntoView {
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
    let addr_res = Resource::new(
        move || id,
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
    );

    let refetch = Callback::new(move |()| {
        addr_res.refetch();
    });

    let on_cancel = use_on_cancel();

    let topic = Signal::derive(move || Some(CrTopic::Address(id)));
    let (version, set_version) = signal(0_u32);
    use_client_registry_socket(topic, version.into(), refetch);

    view! {
        <Transition fallback=move || {
            // Fallback must render a <tr> inside <tbody>, not a <div>.
            view! {
                <tr>
                    <td colspan="2" class="text-center p-4">
                        <span class="loading loading-spinner loading-md"></span>
                    </td>
                </tr>
            }
        }>
            <ErrorBoundary fallback=move |errors| {
                for (_err_id, err) in errors.get().into_iter() {
                    let e = err.into_inner();
                    leptos::logging::log!("Error saving Postal Address: {:?}", e);
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
                    addr_res
                        .and_then(|pa| {
                            set_version.set(pa.get_version().unwrap_or_default());
                            view! { <ShowPostalAddressTableRow id=id pa=pa.clone() /> }
                        })
                }}
            </ErrorBoundary>
        </Transition>
    }
}

#[component]
fn ShowPostalAddressTableRow(id: Uuid, pa: PostalAddress) -> impl IntoView {
    let pa = StoredValue::new(pa);

    assert_eq!(id, pa.read_value().get_id(), "Loaded Postal Address ID does not match the expected ID");

    // navigation helpers
    let UseQueryNavigationReturn {
        url_update_query,
        url_remove_query,
        url_matched_route,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    let address_id = AddressIdQuery::use_param_query();
    let is_selected = move || address_id.get() == Some(id);

    // Callback for updating the selected postal address id, which updates the query string and thus the URL
    let set_selected_id = Callback::new(move |new_id: Option<Uuid>| {
        let nav_url = if let Some(t_id) = new_id {
            url_update_query(AddressIdQuery::KEY, &t_id.to_string())
        } else {
            url_remove_query(AddressIdQuery::KEY)
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

    view! {
        <tr
            class="hover cursor-pointer"
            class:bg-base-200=is_selected
            data-testid=format!("table-entry-row-{}", pa.read_value().get_id())
            on:click=move |_| {
                if address_id.get() == Some(pa.read_value().get_id()) {
                    set_selected_id.run(None);
                } else {
                    set_selected_id.run(Some(pa.read_value().get_id()));
                }
            }
        >
            <td class="font-bold" data-testid=format!("table-entry-name-{}", pa.read_value().get_id())>
                {pa.read_value().get_name().to_string()}
            </td>
            <td data-testid=format!(
                "table-entry-preview-{}",
                pa.read_value().get_id(),
            )>
                {format!(
                    "{} - {}",
                    pa.read_value().get_locality(),
                    pa.read_value().get_country().map(|c| c.name()).unwrap_or_default(),
                )}
            </td>
        </tr>
        <Show when=is_selected>
            <tr>
                <td colspan="2" class="p-0">
                    <div
                        class="flex flex-wrap items-baseline gap-x-2 gap-y-1 p-4 bg-base-200 text-sm"
                        data-testid="table-entry-detailed-preview"
                    >
                        <span data-testid="preview-street" class="font-medium">
                            {pa.read_value().get_street().to_string()}
                        </span>

                        <span class="opacity-50 hidden sm:inline">"•"</span>

                        <span data-testid="preview-postal_locality">
                            <span data-testid="preview-postal_code">
                                {pa.read_value().get_postal_code().to_string()}
                            </span>
                            " "
                            <span data-testid="preview-locality">
                                {pa.read_value().get_locality().to_string()}
                            </span>
                        </span>

                        <Show when=move || pa.read_value().get_region().is_some()>
                            <span class="opacity-50 hidden sm:inline">"•"</span>
                            <span data-testid="preview-region">
                                {pa.read_value().get_region().map(|r| r.to_string()).unwrap_or_default()}
                            </span>
                        </Show>

                        <span class="opacity-50 hidden sm:inline">"•"</span>

                        <span data-testid="preview-country" class="text-base-content/70">
                            {display_country(pa.read_value().get_country())}
                        </span>

                        // Hidden technical fields
                        <span class="hidden" data-testid="preview-address-id">
                            {pa.read_value().get_id().to_string()}
                        </span>
                        <span class="hidden" data-testid="preview-address-version">
                            {pa.read_value().get_version().unwrap_or_default()}
                        </span>
                    </div>
                </td>
            </tr>
            <tr>
                <td colspan="2" class="p-0">
                    <div class="flex gap-2 justify-end p-2 bg-base-200" data-testid="row-actions">
                        <A
                            href=move || url_matched_route(MatchedRouteHandler::Extend("edit"))
                            attr:class="btn btn-sm btn-primary"
                            attr:data-testid="action-btn-edit"
                            scroll=false
                        >
                            "Edit"
                        </A>
                        <A
                            href=move || url_matched_route(MatchedRouteHandler::Extend("copy"))
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
