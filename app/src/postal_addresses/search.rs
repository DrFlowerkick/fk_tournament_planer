//! Postal Address Search Component

use app_core::{CrTopic, PostalAddress};
use app_utils::{
    components::{
        banner::AcknowledgmentAndNavigateBanner,
        set_id_in_query_input_dropdown::{
            SetIdInQueryInputDropdown, SetIdInQueryInputDropdownProperties,
        },
    },
    error::AppError,
    hooks::use_query_navigation::{
        MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
    },
    params::AddressIdQuery,
    server_fn::postal_address::{list_postal_addresses, load_postal_address},
};
use cr_leptos_axum_socket::use_client_registry_socket;
//use cr_single_instance::use_client_registry_sse;
use isocountry::CountryCode;
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_query, nested_router::Outlet};
use uuid::Uuid;

fn display_country(country_code: Option<CountryCode>) -> String {
    country_code
        .map(|c| format!("{} ({})", c.name(), c.alpha2()))
        .unwrap_or_default()
}

#[component]
pub fn SearchPostalAddress() -> impl IntoView {
    // get id from url query parameters & navigation helpers
    let query = use_query::<AddressIdQuery>();
    let UseQueryNavigationReturn {
        url_matched_route,
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();

    // signals for dropdown
    let name = RwSignal::new(String::new());
    let search_text = RwSignal::new(String::new());

    // signals for client registry
    let (id, set_id) = signal(None::<Uuid>);
    let (topic, set_topic) = signal(None::<CrTopic>);
    let (version, set_version) = signal(0_u32);

    // load existing address when query contains address_id
    let addr_res: Resource<Result<PostalAddress, AppError>> = Resource::new(
        move || query.get(),
        move |maybe_id| async move {
            match maybe_id {
                Ok(AddressIdQuery {
                    address_id: Some(id),
                }) => match load_postal_address(id).await {
                    Ok(Some(pa)) => Ok(pa),
                    Ok(None) => Err(AppError::ResourceNotFound("Postal Address".to_string(), id)),
                    Err(e) => Err(e),
                },
                Ok(AddressIdQuery { address_id: None }) => {
                    // no address id: no loading delay
                    Ok(Default::default())
                }
                Err(e) => Err(AppError::Other(e.to_string())),
            }
        },
    );

    let is_addr_res_error = move || matches!(addr_res.get(), Some(Err(_)));

    let refetch = Callback::new(move |()| addr_res.refetch());
    // update address via socket
    use_client_registry_socket(topic.into(), version.into(), refetch);
    // update address via sse
    //use_client_registry_sse(topic, version, refetch);

    // load possible addresses from search_text
    let addr_list = Resource::new(
        move || search_text.get(),
        |name| async move {
            if name.len() > 2 {
                list_postal_addresses(name).await
            } else {
                Ok(vec![])
            }
        },
    );

    let is_addr_list_error = move || matches!(addr_list.get(), Some(Err(_)));

    let is_disabled =
        move || addr_res.get().is_none() || is_addr_res_error() || is_addr_list_error();

    // list of postal addresses matching search_text
    let results = Signal::derive(move || {
        addr_list
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    });

    // reset url when unexpectedly no address found
    let reset_url = move || url_matched_route_remove_query("address_id", MatchedRouteHandler::Keep);

    let props = SetIdInQueryInputDropdownProperties {
        key: "address_id",
        name,
        placeholder: "Enter name of address you are searching...",
        search_text,
        list_items: results,
        render_item: |a| {
            view! {
                <span class="font-medium">{a.get_name().to_string()}</span>
                <span class="text-xs text-base-content/70">
                    {match a.get_region() {
                        Some(region) => {
                            format!(
                                "{} {} · {region} · {}",
                                a.get_postal_code(),
                                a.get_locality(),
                                display_country(a.get_country()),
                            )
                        }
                        None => {
                            format!(
                                "{} {} {}",
                                a.get_postal_code(),
                                a.get_locality(),
                                display_country(a.get_country()),
                            )
                        }
                    }}
                </span>
            }
            .into_any()
        },
    };

    view! {
        <div class="card w-full bg-base-100 shadow-xl" data-testid="search-address">
            <div class="card-body">
                <h2 class="card-title">"Search Postal Address"</h2>
                <Transition fallback=move || {
                    view! {
                        <div class="flex justify-center items-center p-4">
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    }
                }>
                    {move || {
                        addr_res
                            .get()
                            .map(|res| match res {
                                Err(msg) => {
                                    // --- General Load Error Banner ---
                                    view! {
                                        <AcknowledgmentAndNavigateBanner
                                            msg=format!(
                                                "An unexpected error occurred during load: {msg}",
                                            )
                                            ack_btn_text="Reload"
                                            ack_action=move || addr_res.refetch()
                                            nav_btn_text="Reset"
                                            navigate_url=reset_url()
                                        />
                                    }
                                        .into_any()
                                }
                                Ok(addr) => {
                                    name.set(addr.get_name().to_string());
                                    set_version.set(addr.get_version().unwrap_or_default());
                                    if addr.get_version().is_some() {
                                        let new_topic = CrTopic::Address(addr.get_id());
                                        set_topic.set(Some(new_topic));
                                        set_id.set(Some(addr.get_id()));
                                    } else {
                                        set_id.set(None);
                                    }
                                    ().into_any()
                                }
                            })
                    }} <SetIdInQueryInputDropdown props=props />
                    {move || {
                        if let Some(Ok(addr)) = addr_res.get() {
                            view! {
                                <div
                                    class="card w-full bg-base-200 shadow-md mt-4"
                                    data-testid="address-preview"
                                >
                                    <div class="card-body">
                                        <h3 class="card-title" data-testid="preview-address-name">
                                            {addr.get_name().to_string()}
                                        </h3>
                                        <p data-testid="preview-street">
                                            {addr.get_street().to_string()}
                                        </p>
                                        <p data-testid="preview-postal_locality">
                                            <span data-testid="preview-postal_code">
                                                {addr.get_postal_code().to_string()}
                                            </span>
                                            " "
                                            <span data-testid="preview-locality">
                                                {addr.get_locality().to_string()}
                                            </span>
                                        </p>
                                        <p data-testid="preview-region">
                                            {addr.get_region().unwrap_or_default().to_string()}
                                        </p>
                                        <p data-testid="preview-country">
                                            {display_country(addr.get_country())}
                                        </p>
                                        <p class="hidden" data-testid="preview-address-id">
                                            {addr.get_id().to_string()}
                                        </p>
                                        <p class="hidden" data-testid="preview-address-version">
                                            {addr.get_version().unwrap_or_default()}
                                        </p>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    }} <div class="card-actions justify-end mt-4">
                        <A
                            href=move || url_matched_route_remove_query(
                                "address_id",
                                MatchedRouteHandler::Extend("new_pa"),
                            )
                            attr:class="btn btn-primary"
                            attr:data-testid="btn-new-address"
                            attr:disabled=is_disabled
                        >
                            "New"
                        </A>
                        <A
                            href=move || url_matched_route(MatchedRouteHandler::Extend("edit_pa"))
                            attr:class="btn btn-secondary"
                            attr:data-testid="btn-edit-address"
                            attr:disabled=move || is_disabled() || id.get().is_none()
                        >
                            "Edit"
                        </A>
                    </div>
                </Transition>
            </div>
        </div>
        <div class="my-4"></div>
        {if cfg!(not(feature = "test-mock")) {
            view! { <Outlet /> }.into_any()
        } else {
            ().into_any()
        }}
    }
}
