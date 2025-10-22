use super::{
    AddressParams,
    server_fn::{SavePostalAddress, load_postal_address},
};
use app_core::{PaValidationField, PostalAddress};
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

    let id = address.get_id();
    let version = address.get_version();
    let region = address.get_region().unwrap_or_default().to_string();
    let is_new = id.is_none();

    // address signals for normalization and validation
    let (norm_name, set_norm_name) = signal(address.get_name().to_string());
    let (norm_street, set_norm_street) = signal(address.get_street().to_string());
    let (norm_postal_code, set_norm_postal_code) = signal(address.get_postal_code().to_string());
    let (norm_locality, set_norm_locality) = signal(address.get_locality().to_string());
    let (norm_country, set_norm_country) = signal(address.get_country().to_string());
    let (addr, set_addr) = signal(address);
    let is_valid_addr = move || addr.get().validate().is_ok();

    let is_valid_name = move || match addr.get().validate() {
        Ok(()) => true,
        Err(err) => err
            .errors
            .iter()
            .all(|e| *e.get_field() != PaValidationField::Name),
    };

    let is_valid_street = move || match addr.get().validate() {
        Ok(()) => true,
        Err(err) => err
            .errors
            .iter()
            .all(|e| *e.get_field() != PaValidationField::Street),
    };

    let is_valid_postal_code = move || match addr.get().validate() {
        Ok(()) => true,
        Err(err) => err
            .errors
            .iter()
            .all(|e| *e.get_field() != PaValidationField::PostalCode),
    };

    let is_valid_locality = move || match addr.get().validate() {
        Ok(()) => true,
        Err(err) => err
            .errors
            .iter()
            .all(|e| *e.get_field() != PaValidationField::Locality),
    };

    let is_valid_country = move || match addr.get().validate() {
        Ok(()) => true,
        Err(err) => err
            .errors
            .iter()
            .all(|e| *e.get_field() != PaValidationField::Country),
    };

    let navigate = use_navigate();

    let cancel_target = move || {
        id.map(|id| format!("/postal-address/{}", id))
            .unwrap_or_else(|| "/postal-address".to_string())
    };

    view! {
        // Use <ActionForm/> to bind to your save server fn
        <ActionForm action=save_postal_address attr:data-testid="form-address">
            // Hidden meta fields the server expects (id / version / intent)
            <input type="hidden" name="id" prop:value=id.unwrap_or(Uuid::nil()).to_string() />
            <input type="hidden" name="version" prop:value=version.unwrap_or_default() />

            // Disable the whole form while loading existing data
            <fieldset prop:disabled=move || loading>
                // Example: Name
                <label class="block">
                    <span class="block text-sm">"Name"</span>
                    <input
                        class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                        name="name"
                        data-testid="input-name"
                        aria-invalid=move || if is_valid_name() { "false" } else { "true" }
                        // show live value when loaded; while loading show "Loading…" as placeholder
                        prop:value=move || norm_name.get()
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter name..."
                            } else {
                                ""
                            }
                        }
                        on:input=move |ev| {
                            set_addr.write().set_name(event_target_value(&ev));
                        }
                        on:blur=move |_| {
                            set_norm_name.set(addr.get().get_name().to_string());
                        }
                    />
                </label>

                // Street
                <label class="block">
                    <span class="block text-sm">"Street & number"</span>
                    <input
                        class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                        name="street"
                        data-testid="input-street"
                        aria-invalid=move || if is_valid_street() { "false" } else { "true" }
                        prop:value=move || norm_street.get()
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter street and number..."
                            } else {
                                ""
                            }
                        }
                        on:input=move |ev| {
                            set_addr.write().set_street(event_target_value(&ev));
                        }
                        on:blur=move |_| {
                            set_norm_street.set(addr.get().get_street().to_string());
                        }
                    />
                </label>

                // Postal code + City
                <div class="grid grid-cols-2 gap-3">
                    <label class="block">
                        <span class="block text-sm">"Postal code"</span>
                        <input
                            class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                            name="postal_code"
                            data-testid="input-postal_code"
                            aria-invalid=move || {
                                if is_valid_postal_code() { "false" } else { "true" }
                            }
                            prop:value=move || norm_postal_code.get()
                            placeholder=move || {
                                if loading {
                                    "Loading..."
                                } else if is_new {
                                    "Enter postal code..."
                                } else {
                                    ""
                                }
                            }
                            on:input=move |ev| {
                                set_addr.write().set_postal_code(event_target_value(&ev));
                            }
                            on:blur=move |_| {
                                set_norm_postal_code.set(addr.get().get_postal_code().to_string());
                            }
                        />
                    </label>
                    <label class="block">
                        <span class="block text-sm">"City"</span>
                        <input
                            class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                            name="locality"
                            data-testid="input-locality"
                            aria-invalid=move || if is_valid_locality() { "false" } else { "true" }
                            prop:value=move || norm_locality.get()
                            placeholder=move || {
                                if loading {
                                    "Loading..."
                                } else if is_new {
                                    "Enter city..."
                                } else {
                                    ""
                                }
                            }
                            on:input=move |ev| {
                                set_addr.write().set_locality(event_target_value(&ev));
                            }
                            on:blur=move |_| {
                                set_norm_locality.set(addr.get().get_locality().to_string());
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
                        prop:value=region
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
                        class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                        name="country"
                        data-testid="input-country"
                        aria-invalid=move || if is_valid_country() { "false" } else { "true" }
                        prop:value=move || norm_country.get()
                        placeholder=move || {
                            if loading {
                                "Loading..."
                            } else if is_new {
                                "Enter country..."
                            } else {
                                ""
                            }
                        }
                        on:input=move |ev| {
                            set_addr.write().set_country(event_target_value(&ev));
                        }
                        on:blur=move |_| {
                            set_norm_country.set(addr.get().get_country().to_string());
                        }
                    />
                </label>

                // Actions: Update vs. "Save as new"
                <div class="flex gap-2">
                    // Update existing
                    <button
                        type="submit"
                        name="intent"
                        value=move || if is_new {"create"} else {"update"}
                        data-testid="btn-save"
                        class="btn"
                        prop:disabled=move || loading || !is_valid_addr()
                    >
                        "Save"
                    </button>

                    // "save as new" (disabled in "new" mode)
                    <button
                        type="button"
                        name="intent"
                        value="create"
                        data-testid="btn-save-as-new"
                        class="btn"
                        prop:disabled=move || loading || is_new || !is_valid_addr()
                        prop:hidden=move || is_new
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
