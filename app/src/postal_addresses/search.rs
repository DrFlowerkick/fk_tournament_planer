// search for postal address by name

use super::{
    AddressParams,
    server_fn::{list_postal_addresses, load_postal_address},
};
use crate::{
    AppError,
    components::banner::AcknowledgmentAndNavigateBanner,
    global_state::{GlobalState, GlobalStateStoreFields},
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
};
use app_core::{CrTopic, PostalAddress};
use cr_leptos_axum_socket::use_client_registry_socket;
use isocountry::CountryCode;
use std::sync::Arc;
//use cr_single_instance::use_client_registry_sse;
use leptos::{prelude::*, task::spawn_local, web_sys};
use leptos_router::{
    NavigateOptions,
    components::A,
    hooks::{use_navigate, use_query},
    nested_router::Outlet,
};
use reactive_stores::Store;
use uuid::Uuid;

fn display_country(code: &str) -> String {
    CountryCode::for_alpha2(code)
        .map(|c| c.name().to_string())
        .unwrap_or_else(|_| code.to_string()) // Fallback to Code, if invalid
}

#[component]
pub fn SearchPostalAddress() -> impl IntoView {
    // get id from url query parameters & navigation helpers
    let query = use_query::<AddressParams>();
    let UseQueryNavigationReturn {
        update,
        remove,
        relative_sub_url,
        path,
        nav_url,
        ..
    } = use_query_navigation();

    // get global state and set return_after_address_edit
    let state = expect_context::<Store<GlobalState>>();
    let return_after_address_edit = state.return_after_address_edit();
    Effect::watch(
        move || path.get(),
        move |path, prev_path, _| {
            if path.ends_with("new_pa") || path.ends_with("edit_pa") {
                if let Some(prev) = prev_path {
                    if prev.ends_with("new_pa") || prev.ends_with("edit_pa") {
                        // do not update return_after_address_edit when navigating between new/edit forms
                        return;
                    }
                    return_after_address_edit.set(prev.clone());
                } else {
                    let super_path = path
                        .rsplit_once('/')
                        .map(|(p, _)| p)
                        .unwrap_or("/")
                        .to_string();
                    return_after_address_edit.set(super_path);
                }
            }
        },
        true,
    );

    // signals for address fields
    let (name, set_name) = signal(String::new());
    let (id, set_id) = signal(None::<Uuid>);
    let (topic, set_topic) = signal(None::<CrTopic>);
    let (version, set_version) = signal(0_u32);

    // dropdown-status & keyboard-highlight
    let (open, set_open) = signal(false);
    let (hi, set_hi) = signal::<Option<usize>>(None);

    // search_text to search address & loaded / selected address
    let (search_text, set_search_text) = signal(String::new());

    // load existing address when `id` is Some(...)
    let addr_res: Resource<Result<PostalAddress, AppError>> = Resource::new(
        move || query.get(),
        move |maybe_id| async move {
            match maybe_id {
                // AppResult<PostalAddress>
                Ok(AddressParams {
                    address_id: Some(id),
                }) => match load_postal_address(id).await {
                    Ok(Some(pa)) => Ok(pa),
                    Ok(None) => Err(AppError::Generic("Postal Address ID not found".to_string())),
                    Err(e) => Err(e),
                },
                Ok(AddressParams { address_id: None }) => {
                    // no address id: no loading delay
                    Ok(Default::default())
                }
                // new form or bad uuid: no loading delay
                //_ => Ok(Default::default()),
                Err(e) => Err(AppError::Generic(e.to_string())),
            }
        },
    );

    let refetch = Arc::new(move || addr_res.refetch());
    // update address via socket
    use_client_registry_socket(topic, version, refetch);
    // update address via sse
    //use_client_registry_sse(topic, version, refetch);

    let is_addr_res_error = move || matches!(addr_res.get(), Some(Err(_)));

    // these function are required by sse_listener to refetch addr_res after changes to it at server side
    let _refetch = move || addr_res.refetch();
    let _topic = move || id.get().map(CrTopic::Address);

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

    // selection handler
    let select_idx = move |i: usize| {
        if let Some(Ok(list)) = addr_list.get_untracked()
            && let Some(item) = list.get(i)
        {
            // 1) update UI state
            set_search_text.set(item.get_name().to_string());
            set_open.set(false);
            if let Some(Ok(addr)) = addr_res.get()
                && addr.get_id() == item.get_id()
            {
                addr_res.notify();
            } else {
                // 2) update URL
                update("address_id", &item.get_id().unwrap_or_default().to_string());
                let navigate = use_navigate();
                navigate(
                    &nav_url.get(),
                    NavigateOptions {
                        // replace=true prevents „history spam“
                        replace: true,
                        ..Default::default()
                    },
                );
            }
        }
    };

    // keyboard control
    let on_key = move |ev: web_sys::KeyboardEvent| {
        if let Some(Ok(list)) = addr_list.get_untracked() {
            let len = list.len();
            match ev.key().as_str() {
                "ArrowDown" if len > 0 => {
                    ev.prevent_default();
                    let next = hi.get().map(|i| (i + 1) % len).unwrap_or(0);
                    set_hi.set(Some(next));
                    set_open.set(true);
                }
                "ArrowUp" if len > 0 => {
                    ev.prevent_default();
                    let next = hi.get().map(|i| (i + len - 1) % len).unwrap_or(len - 1);
                    set_hi.set(Some(next));
                    set_open.set(true);
                }
                "Enter" => {
                    if let Some(i) = hi.get() {
                        ev.prevent_default();
                        select_idx(i);
                    }
                }
                "Escape" => set_open.set(false),
                _ => {}
            }
        }
    };

    // handle blur
    let on_blur = move |_| {
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            set_open.set(false);
            addr_res.notify();
        });
    };

    // list of postal addresses matching search_text
    let results = move || {
        addr_list
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    };

    // reset url when unepectedly no address found
    let reset_url = move || {
        remove("address_id");
        nav_url.get()
    };

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
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
                                    set_name.set(addr.get_name().to_string());
                                    set_id.set(addr.get_id());
                                    set_version.set(addr.get_version().unwrap_or_default());
                                    if let Some(id) = addr.get_id() {
                                        let new_topic = CrTopic::Address(id);
                                        set_topic.set(Some(new_topic));
                                    }
                                    ().into_any()
                                }
                            })
                    }}
                    <div class=move || {
                        format!("dropdown w-full {}", if open.get() { "dropdown-open" } else { "" })
                    }>
                        <input
                            type="text"
                            class="input input-bordered w-full"
                            prop:value=move || name.get()
                            data-testid="search-input"
                            placeholder="Enter name of address you are searching..."
                            on:input=move |ev| {
                                set_search_text.set(event_target_value(&ev));
                                set_open.set(true);
                                set_hi.set(None);
                            }
                            on:focus=move |_| {
                                if search_text.get().is_empty() {
                                    set_search_text.set(name.get());
                                }
                                set_open.set(true);
                            }
                            on:keydown=on_key
                            on:blur=on_blur
                            autocomplete="off"
                            role="combobox"
                            aria-expanded=move || {
                                if open.get() && !results().is_empty() { "true" } else { "false" }
                            }
                            aria-controls="addr-suggest"
                        />

                        {move || {
                            open.get()
                                .then(|| {
                                    view! {
                                        <ul
                                            id="addr-suggest"
                                            data-testid="search-suggest"
                                            aria-busy=move || {
                                                if results().is_empty() { "true" } else { "false" }
                                            }
                                            class="dropdown-content menu menu-sm bg-base-100 rounded-box z-[1] mt-1 w-full p-0 shadow max-h-72 overflow-auto"
                                            role="listbox"
                                        >
                                            {move || {
                                                if results().is_empty() {
                                                    view! {
                                                        <li class="px-3 py-2 text-sm text-base-content/70">
                                                            "Searching…"
                                                        </li>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <For
                                                            each=move || results().clone().into_iter().enumerate()
                                                            key=|(_i, a)| a.get_id_version()
                                                            children=move |(i, a)| {
                                                                let is_hi = move || {
                                                                    hi.get().map(|j| j == i).unwrap_or(false)
                                                                };
                                                                let opt_id = format!("addr-option-{}", i);

                                                                view! {
                                                                    <li
                                                                        id=opt_id.clone()
                                                                        data-testid="search-suggest-item"
                                                                        role="option"
                                                                        aria-selected=move || if is_hi() { "true" } else { "false" }
                                                                        class:active=move || is_hi()
                                                                    >
                                                                        <a
                                                                            class="flex flex-col items-start gap-0.5"
                                                                            class:active=move || is_hi()
                                                                            class:bg-base-200=move || is_hi()
                                                                            on:mouseenter=move |_| set_hi.set(Some(i))
                                                                            on:mousedown=move |_| select_idx(i)
                                                                        >
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
                                                                        </a>
                                                                    </li>
                                                                }
                                                            }
                                                        />
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        </ul>
                                    }
                                })
                        }}
                    </div>
                    {move || {
                        if let Some(Ok(addr)) = addr_res.get() {
                            if addr.get_id().is_some() {
                                view! {
                                    <div
                                        class="card w-full bg-base-200 shadow-md mt-4"
                                        data-testid="address-preview"
                                    >
                                        <div class="card-body">
                                            <h3 class="card-title" data-testid="preview-name">
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
                                                {display_country(&addr.get_country())}
                                            </p>
                                            <p class="hidden" data-testid="preview-id">
                                                {addr.get_id().unwrap_or_default().to_string()}
                                            </p>
                                            <p class="hidden" data-testid="preview-version">
                                                {addr.get_version().unwrap_or_default()}
                                            </p>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="mt-4">
                                        <p>"No address selected."</p>
                                    </div>
                                }
                                    .into_any()
                            }
                        } else {
                            ().into_any()
                        }
                    }} <div class="card-actions justify-end mt-4">
                        <A
                            href=move || relative_sub_url("new_pa")
                            attr:class="btn btn-primary"
                            attr:data-testid="btn-new-address"
                            attr:disabled=is_disabled
                        >
                            "New"
                        </A>
                        <A
                            href=move || relative_sub_url("edit_pa")
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
