// web ui for adding and modifying postal addresses

use crate::AppResult;
use app_core::{CrTopic, PostalAddress};
use cr_single_instance::{use_changed_sse, SseUrl};
use leptos::{prelude::*, task::spawn_local, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_params},
    params::Params,
};
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
struct AddressParams {
    pub uuid: Option<Uuid>,
}

#[component]
pub fn NewPostalAddress() -> impl IntoView {
    view! { <AddressFormWrapper id=None /> }
}

#[component]
pub fn PostalAddressEdit() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();
    let id = params.get().map(|ap| ap.uuid).unwrap_or(None);
    view! { <AddressFormWrapper id=id /> }
}

#[component]
pub fn SearchPostalAddress() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();
    let id = move || params.get().map(|ap| ap.uuid).unwrap_or(None);

    // dropdown-status & keyboard-highlight
    let (open, set_open) = signal(false);
    let (hi, set_hi) = signal::<Option<usize>>(None);

    // query to search address & loaded / selected address
    let (query, set_query) = signal(String::new());
    let (address, set_address) = signal(PostalAddress::default());

    // load existing address when `id` is Some(...)
    let addr_res = Resource::new(
        move || id(),
        move |maybe_id| async move {
            match maybe_id {
                // AppResult<PostalAddress>
                Some(id) => load_postal_address(id).await,
                // new form: no loading delay
                None => Ok(Default::default()),
            }
        },
    );

    let refetch = move || addr_res.refetch();
    let version = move || address.get().version;
    // use id from address, since this address is either an existing postal address or nil
    // id from use_params() may be broken or not existing id
    let topic_url = move || CrTopic::Address(address.get().id).sse_url();
    #[cfg(all(target_arch="wasm32", feature="hydrate"))]
    use_changed_sse(CrTopic::Address(address.get().id), refetch, version);

    // initialize query and address from Uuid
    Effect::new(move || {
        if let Some(Ok(addr)) = addr_res.get() {
            // Passe diese Felder an deinen Typ an:
            set_address.set(addr.clone());
            set_query.set(addr.name.clone().unwrap_or_default());
        }
    });

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
        if let Some(Ok(list)) = addr_list.get_untracked() {
            if let Some(item) = list.get(i) {
                // 1) update UI state
                set_query.set(item.name.clone().unwrap_or_default());
                set_address.set(item.clone());
                set_open.set(false);

                // 2) update URL aktualisieren
                let id_str = item.id.to_string(); // falls id: Uuid
                let navigate = use_navigate();
                let _ = navigate(
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
        let set_open = set_open.clone();
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            set_open.set(false);
        });
    };

    let loading = move || addr_list.get().is_none();
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
                    <p> {move || topic_url() }</p>
                    // DaisyUI dropdown container
                    <div class=move || {
                        format!("dropdown w-full {}", if open.get() { "dropdown-open" } else { "" })
                    }>
                        // input for name
                        <input
                            type="text"
                            class="input input-bordered w-full"
                            prop:value=move || query.get()
                            placeholder="Enter name of address you are searching..."
                            on:input=move |ev| {
                                set_query.set(event_target_value(&ev));
                                set_open.set(true);
                                set_hi.set(None);
                            }
                            on:focus=move |_| set_open.set(true)
                            on:keydown=on_key
                            on:blur=on_blur
                            autocomplete="off"
                            role="combobox"
                            aria-expanded=move || {
                                if open.get() && !results().is_empty() { "true" } else { "false" }
                            }
                            aria-controls="addr-suggest"
                        />

                        // optional loading indicator at right-bottom, only visible if loading
                        {move || {
                            loading()
                                .then(|| {
                                    view! {
                                        <span class="loading loading-spinner loading-sm absolute right-3 top-3"></span>
                                    }
                                })
                        }}

                        // dropdown list
                        {move || {
                            (open.get() && (!results().is_empty() || loading()))
                                .then(|| {
                                    view! {
                                        <ul
                                            id="addr-suggest"
                                            class="dropdown-content menu menu-sm bg-base-100 rounded-box z-[1] mt-1 w-full p-0 shadow max-h-72 overflow-auto"
                                            role="listbox"
                                        >
                                            {move || {
                                                if loading() {
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
                                                            key=|(_i, a)| a.id
                                                            children=move |(i, a)| {
                                                                let is_hi = move || {
                                                                    hi.get().map(|j| j == i).unwrap_or(false)
                                                                };
                                                                let opt_id = format!("addr-option-{}", i);
                                                                // for a11y

                                                                view! {
                                                                    <li
                                                                        id=opt_id.clone()
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
                                                                            <span class="font-medium">
                                                                                {a.name.clone().unwrap_or_default()}
                                                                            </span>
                                                                            <span class="text-xs text-base-content/70">
                                                                                {format!(
                                                                                    "{} {} · {:?} · {}",
                                                                                    a.postal_code,
                                                                                    a.address_locality,
                                                                                    a.address_region,
                                                                                    a.address_country,
                                                                                )}
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
                    <div class="mt-3 space-y-1 text-sm">
                        <p>{move || address.get().street_address}</p>
                        <p>
                            {move || {
                                format!(
                                    "{} {}",
                                    address.get().postal_code,
                                    address.get().address_locality,
                                )
                            }}
                        </p>
                        <p>{move || address.get().address_region}</p>
                        <p>{move || address.get().address_country}</p>
                    </div>

                    <div class="mt-4 flex gap-2">
                        // NEW: always clickable
                        <a href="/postal-address/new" class="btn btn-primary btn-sm">
                            "New"
                        </a>

                        // MODIFY: only active, if valid address is selected
                        <button
                            class="btn btn-secondary btn-sm"
                            disabled=move || address.get().id.is_nil()
                            on:click=move |_| {
                                let id = address.get().id;
                                let navigate = use_navigate();
                                let _ = navigate(
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

#[component]
pub fn AddressFormWrapper(id: Option<Uuid>) -> impl IntoView {
    // Resource: load existing address when `id` is Some(...)
    let addr_res = Resource::new(
        move || id,
        |maybe_id| async move {
            match maybe_id {
                // AppResult<PostalAddress>
                Some(id) => load_postal_address(id).await,
                // new form: no loading delay
                None => Ok(Default::default()),
            }
        },
    );

    view! {
        <Transition fallback=move || {
            view! { <AddressForm address=PostalAddress::default() loading=true /> }
        }>
            {move || {
                addr_res
                    .get()
                    .map(|a| view! { <AddressForm address=a.unwrap_or_default() loading=false /> })
            }}
        </Transition>
    }
}

#[component]
pub fn AddressForm(address: PostalAddress, loading: bool) -> impl IntoView {
    let save_postal_address = ServerAction::<SavePostalAddress>::new();

    let is_new = address.version == -1 || address.id.is_nil();

    view! {
        // Use <ActionForm/> to bind to your save server fn
        <ActionForm action=save_postal_address>
            // Hidden meta fields the server expects (id / version / intent)
            <input type="hidden" name="id" prop:value=address.id.to_string() />
            <input type="hidden" name="version" prop:value=address.version />

            // Disable the whole form while loading existing data
            <fieldset prop:disabled=move || loading>
                // Example: Name
                <label class="block">
                    <span class="block text-sm">"Name (optional)"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="name"
                        // show live value when loaded; while loading show "Loading…" as placeholder
                        prop:value=address.name.unwrap_or_default()
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter name (optional)..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Street
                <label class="block">
                    <span class="block text-sm">"Street & number"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="street_address"
                        prop:value=address.street_address
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter street and number..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Postal code + City
                <div class="grid grid-cols-2 gap-3">
                    <label class="block">
                        <span class="block text-sm">"Postal code"</span>
                        <input
                            class="w-full border rounded p-2"
                            name="postal_code"
                            prop:value=address.postal_code
                            placeholder=move || {
                                if loading {
                                    "Loading..."
                                } else if is_new {
                                    "Enter postal code..."
                                } else {
                                    ""
                                }
                            }
                        />
                    </label>
                    <label class="block">
                        <span class="block text-sm">"City"</span>
                        <input
                            class="w-full border rounded p-2"
                            name="address_locality"
                            prop:value=address.address_locality
                            placeholder=move || {
                                if loading {
                                    "Loading..."
                                } else if is_new {
                                    "Enter city..."
                                } else {
                                    ""
                                }
                            }
                        />
                    </label>
                </div>

                // Region (optional)
                <label class="block">
                    <span class="block text-sm">"Region (optional)"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="address_region"
                        prop:value=address.address_region
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter region (optional)..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Country
                <label class="block">
                    <span class="block text-sm">"Country (ISO/name)"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="address_country"
                        prop:value=address.address_country
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter country..."
                            } else {
                                ""
                            }
                        }
                    />
                </label>

                // Actions: Update vs. "Save as new"
                <div class="flex gap-2">
                    // Update existing (disabled in "new" mode)
                    <button
                        type="submit"
                        name="intent"
                        value="update"
                        class="rounded px-4 py-2 border"
                        prop:disabled=move || loading || is_new
                        prop:hidden=move || is_new
                    >
                        "Save"
                    </button>

                    // Always allow "save as new"
                    <button
                        type="submit"
                        name="intent"
                        value="create"
                        class="rounded px-4 py-2 border"
                        prop:disabled=move || loading
                    >
                        "Save as new"
                    </button>
                </div>
            </fieldset>
        </ActionForm>
    }
}

#[server]
pub async fn load_postal_address(id: Uuid) -> AppResult<PostalAddress> {
    // get core from context
    use app_core::CoreState;
    let mut core = expect_context::<CoreState>().as_postal_address_state();
    let pa = if let Some(pa) = core.load(id).await? {
        pa.to_owned()
    } else {
        PostalAddress::default()
    };
    Ok(pa)
}

#[server]
pub async fn save_postal_address(
    // hidden in the form; nil => new; else => update
    id: Uuid,
    // hidden in the form; -1 => new; else => update
    version: i64,
    // optional text field: treat "" as None
    name: Option<String>,
    street_address: String,
    postal_code: String,
    address_locality: String,
    // optional text field: treat "" as None
    address_region: Option<String>,
    address_country: String,
    // which submit button was clicked: "update" | "create"
    intent: Option<String>,
) -> AppResult<()> {
    // get core from context
    use app_core::CoreState;
    let mut core = expect_context::<CoreState>().as_postal_address_state();

    if matches!(intent.as_deref(), Some("update")) {
        // set id and version previously loaded
        core.set_id(id);
        core.set_version(version);
    }

    let name = name.unwrap_or_default();
    core.change_name(name);
    core.change_street_address(street_address);
    core.change_postal_code(postal_code);
    core.change_address_locality(address_locality);
    let address_region = address_region.unwrap_or_default();
    core.change_address_region(address_region);
    core.change_address_country(address_country);

    // ToDo: gracefully handle errors, e.g. retry
    let saved = core.save().await?;
    let route = format!("/postal-address/{}", saved.id);
    // redirect to newly saved postal address
    leptos_axum::redirect(&route);
    Ok(())
}

#[server]
pub async fn list_postal_addresses(name: String) -> AppResult<Vec<PostalAddress>> {
    // get core from context
    use app_core::CoreState;
    let core = expect_context::<CoreState>().as_postal_address_state();
    let list = core.list_addresses(Some(&name), Some(10)).await?;
    Ok(list)
}
