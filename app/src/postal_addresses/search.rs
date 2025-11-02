// search for postal address by name

use std::sync::Arc;

use super::{
    AddressParams,
    server_fn::{list_postal_addresses, load_postal_address},
};
use crate::{AppError, banner::AcknowledgmentAndNavigateBanner};
use app_core::{CrMsg, CrTopic, PostalAddress};
use cr_leptos_axum_socket::use_client_registry_topic;
use cr_single_instance::SseUrl;
use leptos::{prelude::*, task::spawn_local, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_params},
};
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use uuid::Uuid;

#[component]
pub fn SearchPostalAddress() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();

    // signals for address fields
    let (name, set_name) = signal(String::new());
    let (id, set_id) = signal(None::<Uuid>);
    let (topic, set_topic) = signal(None::<CrTopic>);
    let (version, set_version) = signal(0_u32);
    let (_url, set_url) = signal(String::new());

    // setup sse listener
    /*let UseEventSourceReturn {
        data,
        ready_state,
        ..
    } = use_event_source_with_options::<CrMsg, JsonSerdeCodec>(
        url,
        UseEventSourceOptions::default()
            .immediate(false)
            .named_events(["changed".to_string()]),
    );*/

    let (_data, _set_data) = signal(None::<CrMsg>);

    // dropdown-status & keyboard-highlight
    let (open, set_open) = signal(false);
    let (hi, set_hi) = signal::<Option<usize>>(None);

    // query to search address & loaded / selected address
    let (query, set_query) = signal(String::new());

    // load existing address when `id` is Some(...)
    let addr_res: Resource<Result<PostalAddress, AppError>> = Resource::new(
        move || params.get(),
        move |maybe_id| async move {
            let navigate = use_navigate();
            match maybe_id {
                // AppResult<PostalAddress>
                Ok(AddressParams { uuid: Some(id) }) => match load_postal_address(id).await {
                    Ok(Some(pa)) => Ok(pa),
                    Ok(None) => {
                        navigate(
                            "/postal-address",
                            NavigateOptions {
                                replace: true,
                                ..Default::default()
                            },
                        );
                        Ok(Default::default())
                    }
                    Err(_e) => Ok(Default::default()),
                    //Err(e) => Err(e),
                },
                // new form or bad uuid: no loading delay
                _ => Ok(Default::default()),
            }
        },
    );

    /*Effect::new(move || {
        if let Some(event) = data.get() {
            match event {
                CrMsg::AddressUpdated { version: meta_version, .. } => {
                    if meta_version > version.get_untracked() {
                        addr_res.refetch();
                    }
                }
            }
        }
    });*/

    let refetch = Arc::new(move || addr_res.refetch());
    use_client_registry_topic(topic, version, refetch);

    let is_addr_res_error = move || matches!(addr_res.get(), Some(Err(_)));

    // these function are required by sse_listener to refetch addr_res after changes to it at server side
    let _refetch = move || addr_res.refetch();
    let _topic = move || id.get().map(CrTopic::Address);

    // load possible addresses from query
    let addr_list = Resource::new(
        move || query.get(),
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
            set_query.set(item.get_name().to_string());
            set_open.set(false);
            if let Some(Ok(addr)) = addr_res.get()
                && addr.get_id() == item.get_id()
            {
                addr_res.notify();
            } else {
                // 2) update URL
                let id_str = item.get_id().map(|id| id.to_string()).unwrap_or_default();
                let navigate = use_navigate();
                navigate(
                    &format!("/postal-address/{}", id_str),
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

    let results = move || {
        addr_list
            .get()
            .map(|res| res.unwrap_or_default())
            .unwrap_or_default()
    };

    view! {
        <Transition fallback=move || {
            view! {
                <div>
                    <p>"Searching for address..."</p>
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
                                    msg=format!("An unexpected error occurred during load: {msg}")
                                    ack_btn_text="Reload"
                                    ack_action=move || addr_res.refetch()
                                    nav_btn_text="Reset"
                                    navigate_url="/postal-address".into()
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
                                set_url.set(new_topic.sse_url());
                                set_topic.set(Some(new_topic));
                            }
                            ().into_any()
                        }
                    })
            }} // DaisyUI dropdown container
            <div class=move || {
                format!("dropdown w-full {}", if open.get() { "dropdown-open" } else { "" })
            }>
                // input for name
                <input
                    type="text"
                    class="input input-bordered w-full"
                    prop:value=move || name.get()
                    data-testid="search-input"
                    placeholder="Enter name of address you are searching..."
                    on:input=move |ev| {
                        set_query.set(event_target_value(&ev));
                        set_open.set(true);
                        set_hi.set(None);
                    }
                    on:focus=move |_| {
                        if query.get().is_empty() {
                            set_query.set(name.get());
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

                // dropdown list
                {move || {
                    open.get()
                        .then(|| {
                            view! {
                                <ul
                                    id="addr-suggest"
                                    data-testid="search-suggest"
                                    // aria-busy=true while loading resource, otherwise false
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
                                                        // for a11y

                                                        view! {
                                                            <li
                                                                id=opt_id.clone()
                                                                data-testid="search-suggest-item"
                                                                role="option"
                                                                // a11y: mark current „active“ option element
                                                                aria-selected=move || if is_hi() { "true" } else { "false" }
                                                                class:active=move || is_hi()
                                                            >
                                                                <a
                                                                    class="flex flex-col items-start gap-0.5"
                                                                    class:active=move || is_hi()
                                                                    class:bg-base-200=move || is_hi()
                                                                    on:mouseenter=move |_| set_hi.set(Some(i))
                                                                    // before blur
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
                                                                                    a.get_country(),
                                                                                )
                                                                            }
                                                                            None => {
                                                                                format!(
                                                                                    "{} {} {}",
                                                                                    a.get_postal_code(),
                                                                                    a.get_locality(),
                                                                                    a.get_country(),
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
            // current selected address
            </div>
            {move || {
                if let Some(Ok(addr)) = addr_res.get() {
                    view! {
                        <div class="mt-3 space-y-1 text-sm" data-testid="address-preview">
                            <h2 data-testid="preview-name">{addr.get_name().to_string()}</h2>
                            <p data-testid="preview-street">{addr.get_street().to_string()}</p>
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
                            <p data-testid="preview-country">{addr.get_country().to_string()}</p>
                            <p class="hidden" data-testid="preview-id">
                                {addr.get_id().unwrap_or_default().to_string()}
                            </p>
                            //<p class="hidden" data-testid="preview-version">
                            <p data-testid="preview-version">
                                {addr.get_version().unwrap_or_default()}
                            </p>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }} // --- Action Buttons ---
            <div class="mt-4 flex gap-2">
                // NEW: always clickable (if now error)
                <button
                    class="btn btn-secondary btn-sm"
                    data-testid="btn-new-address"
                    disabled=is_disabled
                    on:click=move |_| {
                        let navigate = use_navigate();
                        navigate("/postal-address/new", NavigateOptions::default());
                    }
                >
                    "New"
                </button>

                // MODIFY: only active, if valid address is selected and no error
                <button
                    class="btn btn-secondary btn-sm"
                    data-testid="btn-modify-address"
                    disabled=move || is_disabled() || id.get().is_none()
                    on:click=move |_| {
                        let id = id
                            .get()
                            .expect(
                                "Save expect, since id.get() returns Some(). Otherwise button would be disabled.",
                            );
                        let navigate = use_navigate();
                        navigate(&format!("/postal-address/{id}/edit"), NavigateOptions::default());
                    }
                >
                    "Modify"
                </button>
            </div>
        </Transition>
    }
}
