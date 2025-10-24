// search for postal address by name

use super::{
    AddressParams,
    server_fn::{list_postal_addresses, load_postal_address},
};
use crate::SseListener;
use app_core::CrTopic;
use leptos::{prelude::*, task::spawn_local, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_params},
};
use uuid::Uuid;

#[component]
pub fn SearchPostalAddress() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();
    let (id, set_id) = signal(None::<Uuid>);

    Effect::new(move |_| {
        set_id.set(params.get().ok().and_then(|p| p.uuid));
    });

    // dropdown-status & keyboard-highlight
    let (open, set_open) = signal(false);
    let (hi, set_hi) = signal::<Option<usize>>(None);

    // query to search address & loaded / selected address
    let (query, set_query) = signal(String::new());

    // load existing address when `id` is Some(...)
    let addr_res = Resource::new(
        move || id.get(),
        move |maybe_id| async move {
            match maybe_id {
                // AppResult<PostalAddress>
                Some(id) => load_postal_address(id).await.unwrap_or_default(),
                // new form: no loading delay
                None => Default::default(),
            }
        },
    );

    // these function are required by sse_listener to refetch addr_res after changes to it at server side
    let refetch = move || addr_res.refetch();
    let version = move || {
        addr_res
            .get()
            .map(|a| a.get_version().unwrap_or_default())
            .unwrap_or_default()
    };
    // use id from addr_res, since it provides either an existing postal address id or None
    // id from use_params() may be broken or not existing id
    let topic = move || {
        addr_res
            .get()
            .and_then(|a| a.get_id().map(CrTopic::Address))
    };

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

    // selection handler
    let select_idx = move |i: usize| {
        if let Some(Ok(list)) = addr_list.get_untracked()
            && let Some(item) = list.get(i)
        {
            // 1) update UI state
            set_query.set(item.get_name().to_string());
            if id.get() == item.get_id() {
                addr_res.notify();
            } else {
                set_id.set(item.get_id());
            }
            set_open.set(false);

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
                view! {
                    // Only call sse listener, if some topic exists
                    {move || match topic() {
                        Some(topic) => {
                            view! { <SseListener topic=topic version=version refetch=refetch /> }
                                .into_any()
                        }
                        None => {
                            view! {
                                <p class="hidden" data-testid="sse-status">
                                    "disconnected"
                                </p>
                            }
                                .into_any()
                        }
                    }}

                    // DaisyUI dropdown container
                    <div class=move || {
                        format!("dropdown w-full {}", if open.get() { "dropdown-open" } else { "" })
                    }>
                        // input for name
                        <input
                            type="text"
                            class="input input-bordered w-full"
                            prop:value=move || addr_res.get().map(|a| a.get_name().to_string())
                            data-testid="search-input"
                            placeholder="Enter name of address you are searching..."
                            on:input=move |ev| {
                                set_query.set(event_target_value(&ev));
                                set_open.set(true);
                                set_hi.set(None);
                            }
                            on:focus=move |_| {
                                if query.get().is_empty() {
                                    set_query
                                        .set(
                                            addr_res
                                                .get()
                                                .map(|a| a.get_name().to_string())
                                                .unwrap_or_default(),
                                        );
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
                                                if results().is_empty() {
                                                    "true"
                                                } else {
                                                    "false"
                                                }
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
                    </div>

                    // current selected address
                    <div class="mt-3 space-y-1 text-sm" data-testid="address-preview">
                        <h2 data-testid="preview-name">
                            {move || addr_res.get().map(|a| a.get_name().to_string())}
                        </h2>
                        <p data-testid="preview-street">
                            {move || addr_res.get().map(|a| a.get_street().to_string())}
                        </p>
                        <p data-testid="preview-postal_locality">
                            <span data-testid="preview-postal_code">
                                {move || addr_res.get().map(|a| a.get_postal_code().to_string())}

                            </span>
                            " "
                            <span data-testid="preview-locality">
                                {move || addr_res.get().map(|a| a.get_locality().to_string())}
                            </span>
                        </p>
                        <p data-testid="preview-region">
                            {move || {
                                addr_res
                                    .get()
                                    .map(|a| a.get_region().unwrap_or_default().to_string())
                            }}
                        </p>
                        <p data-testid="preview-country">
                            {move || addr_res.get().map(|a| a.get_country().to_string())}
                        </p>
                        <p class="hidden" data-testid="preview-id">
                            {move || {
                                addr_res.get().map(|a| a.get_id().unwrap_or_default().to_string())
                            }}
                        </p>
                        <p class="hidden" data-testid="preview-version">
                            {move || addr_res.get().map(|a| a.get_version().unwrap_or_default())}
                        </p>
                    </div>

                    <div class="mt-4 flex gap-2">
                        // NEW: always clickable
                        <a
                            href="/postal-address/new"
                            class="btn btn-primary btn-sm"
                            data-testid="btn-new-address"
                        >
                            "New"
                        </a>

                        // MODIFY: only active, if valid address is selected
                        <button
                            class="btn btn-secondary btn-sm"
                            data-testid="btn-modify-address"
                            disabled=move || addr_res.get().and_then(|a| a.get_id()).is_none()
                            on:click=move |_| {
                                let id = addr_res
                                    .get()
                                    .expect(
                                        "Save expect, since get_id() returns Some(). Otherwise button would be disabled.",
                                    )
                                    .get_id()
                                    .expect(
                                        "Save expect, since get_id() returns Some(). Otherwise button would be disabled.",
                                    );
                                let navigate = use_navigate();
                                navigate(
                                    &format!("/postal-address/{id}/edit"),
                                    NavigateOptions {
                                        replace: false,
                                        ..Default::default()
                                    },
                                );
                            }
                        >
                            "Modify"
                        </button>
                    </div>
                }
            }}
        </Transition>
    }
}
