use super::{
    AddressParams,
    server_fn::{SavePostalAddress, load_postal_address},
};
use app_core::PostalAddress;
// this is probably in most cases only used for debugging. Remove it if not used anymore.
use leptos::logging::log;
use leptos::{leptos_dom::helpers::set_timeout, prelude::*, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_params},
};
use uuid::Uuid;

#[component]
pub fn NewPostalAddress() -> impl IntoView {
    view! { <AddressFormWrapper id=None /> }
}

#[component]
pub fn PostalAddressEdit() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();
    let id = params.get_untracked().map(|ap| ap.uuid).unwrap_or(None);
    view! { <AddressFormWrapper id=id /> }
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
    // ToDo: use these signal for validation!
    let (_addr, _set_addr) = signal(address.clone());

    let id = address.get_id();
    let is_new = id.is_none();

    let navigate = use_navigate();

    let cancel_target = move || {
        id.map(|id| format!("/postal-address/{}", id))
            .unwrap_or_else(|| "/postal-address".to_string())
    };

    view! {
        // Use <ActionForm/> to bind to your save server fn
        <ActionForm action=save_postal_address attr:data-testid="form-address">
            // Hidden meta fields the server expects (id / version / intent)
            <input
                type="hidden"
                name="id"
                prop:value=address.get_id().unwrap_or(Uuid::nil()).to_string()
            />
            <input
                type="hidden"
                name="version"
                prop:value=address.get_version().unwrap_or_default()
            />

            // Disable the whole form while loading existing data
            <fieldset prop:disabled=move || loading>
                // Example: Name
                <label class="block">
                    <span class="block text-sm">"Name (optional)"</span>
                    <input
                        class="w-full border rounded p-2"
                        name="name"
                        data-testid="input-name"
                        // show live value when loaded; while loading show "Loading…" as placeholder
                        prop:value=address.get_name().unwrap_or_default().to_string()
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
                        name="street"
                        data-testid="input-street"
                        prop:value=address.get_street().to_string()
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
                            data-testid="input-postal_code"
                            prop:value=address.get_postal_code().to_string()
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
                            name="locality"
                            data-testid="input-locality"
                            prop:value=address.get_locality().to_string()
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
                        name="region"
                        data-testid="input-region"
                        prop:value=address.get_region().map(|ar| ar.to_string())
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
                        name="country"
                        data-testid="input-country"
                        prop:value=address.get_country().to_string()
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
                        data-testid="btn-save"
                        class="btn"
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
                        data-testid="btn-save-as-new"
                        class="btn"
                        prop:disabled=move || loading
                    >
                        "Save as new"
                    </button>

                    <button
                        type="button"
                        name="intent"
                        value="cancel"
                        data-testid="btn-cancel"
                        class="btn"
                        on:click=move |_| {
                            let Some(win) = web_sys::window() else {
                                navigate(&cancel_target(), NavigateOptions::default());
                                return;
                            };
                            let before = win.location().href().unwrap_or_default();
                            let used_back = win
                                .history()
                                .ok()
                                .and_then(|h| h.length().ok())
                                .map(|len| {
                                    if len > 1 {
                                        log!("Moving back in history");
                                        let _ = win.history().unwrap().back();
                                        true
                                    } else {
                                        false
                                    }
                                })
                                .unwrap_or(false);
                            let nav = navigate.clone();
                            let target = cancel_target();
                            set_timeout(
                                move || {
                                    if let Some(win2) = web_sys::window() {
                                        let after = win2.location().href().unwrap_or_default();
                                        if !used_back || after == before {
                                            nav(&target, NavigateOptions::default());
                                        }
                                    } else {
                                        nav(&target, NavigateOptions::default());
                                    }
                                },
                                std::time::Duration::from_millis(300),
                            );
                        }
                    >
                        "Cancel"
                    </button>
                </div>
            </fieldset>
        </ActionForm>
    }
}
