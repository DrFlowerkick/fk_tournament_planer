use super::{
    AddressParams,
    server_fn::{SavePostalAddress, load_postal_address},
};
use crate::{
    AppError,
    banner::{AcknowledgmentAndNavigateBanner, AcknowledgmentBanner},
};
use app_core::{PaValidationField, PostalAddress};
use leptos::{leptos_dom::helpers::set_timeout, prelude::*, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_params},
};
use uuid::Uuid;

#[component]
pub fn NewPostalAddress() -> impl IntoView {
    view! { <AddressForm id=None /> }
}

#[component]
pub fn PostalAddressEdit() -> impl IntoView {
    // get id from url
    let params = use_params::<AddressParams>();
    let id = params.get_untracked().map(|ap| ap.uuid).unwrap_or(None);
    view! { <AddressForm id=id /> }
}

// Wrapper component to provide type safe refetch function via context
#[component]
pub fn AddressForm(id: Option<Uuid>) -> impl IntoView {
    let navigate = use_navigate();

    // --- Server Actions & Resources ---
    let save_postal_address = ServerAction::<SavePostalAddress>::new();
    let addr_res = Resource::new(
        move || id,
        |maybe_id| async move {
            match maybe_id {
                Some(id) => match load_postal_address(id).await {
                    Ok(Some(addr)) => Ok(addr),
                    Ok(None) => Err(AppError::Db("Not found".to_string())),
                    Err(e) => Err(e),
                },
                None => Ok(Default::default()),
            }
        },
    );

    let refetch_and_reset = move || {
        addr_res.refetch();
        save_postal_address.clear();
    };

    // --- Signals for form fields ---
    let (name, set_name) = signal(String::new());
    let (street, set_street) = signal(String::new());
    let (postal_code, set_postal_code) = signal(String::new());
    let (locality, set_locality) = signal(String::new());
    let (region, set_region) = signal(String::new());
    let (country, set_country) = signal(String::new());
    let (version, set_version) = signal(0);

    // --- Signals for UI state & errors ---
    let pending = save_postal_address.pending();
    let is_new = id.is_none();

    // --- Derived Signals for Error States ---
    // reset these signals with save_postal_address.clear() when needed
    let is_conflict = move || {
        if let Some(Err(AppError::Db(ref msg))) = save_postal_address.value().get() {
            msg.contains("optimistic lock conflict")
        } else {
            false
        }
    };
    let is_duplicate = move || {
        if let Some(Err(AppError::Db(ref msg))) = save_postal_address.value().get() {
            msg.contains("unique violation")
        } else {
            false
        }
    };
    let is_addr_res_error = move || matches!(addr_res.get(), Some(Err(_)));
    let is_general_error = move || {
        if let Some(Err(err)) = save_postal_address.value().get() {
            match err {
                AppError::Db(ref msg) => {
                    if msg.contains("optimistic lock conflict") || msg.contains("unique violation")
                    {
                        None
                    } else {
                        Some(msg.clone())
                    }
                }
                _ => Some(format!("{:?}", err)),
            }
        } else {
            None
        }
    };

    let is_disabled = move || {
        addr_res.get().is_none()
            || pending.get()
            || is_conflict()
            || is_duplicate()
            || is_addr_res_error()
            || is_general_error().is_some()
    };

    // --- Derived Signal for Validation & Normalization ---
    let current_address = move || {
        let mut addr = PostalAddress::default();
        addr.set_name(name.get());
        addr.set_street(street.get());
        addr.set_postal_code(postal_code.get());
        addr.set_locality(locality.get());
        addr.set_country(country.get());
        if !region.get().is_empty() {
            addr.set_region(region.get());
        }
        addr
    };

    let validation_result = move || current_address().validate();
    let is_valid_addr = move || validation_result().is_ok();

    // --- Simplified Validation Closures ---
    let is_field_valid = move |field: PaValidationField| match validation_result() {
        Ok(_) => true,
        Err(err) => err.errors.iter().all(|e| *e.get_field() != field),
    };

    let is_valid_name = move || is_field_valid(PaValidationField::Name);
    let is_valid_street = move || is_field_valid(PaValidationField::Street);
    let is_valid_postal_code = move || is_field_valid(PaValidationField::PostalCode);
    let is_valid_locality = move || is_field_valid(PaValidationField::Locality);
    let is_valid_country = move || is_field_valid(PaValidationField::Country);

    let cancel_target = move || {
        id.map(|id| format!("/postal-address/{}", id))
            .unwrap_or_else(|| "/postal-address".to_string())
    };

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
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
                                    ack_action=refetch_and_reset
                                    nav_btn_text="Cancel"
                                    navigate_url=cancel_target()
                                />
                            }
                                .into_any()
                        }
                        Ok(addr) => {
                            set_name.set(addr.get_name().to_string());
                            set_street.set(addr.get_street().to_string());
                            set_postal_code.set(addr.get_postal_code().to_string());
                            set_locality.set(addr.get_locality().to_string());
                            set_region.set(addr.get_region().unwrap_or_default().to_string());
                            set_country.set(addr.get_country().to_string());
                            set_version.set(addr.get_version().unwrap_or_default());
                            ().into_any()
                        }
                    })
            }} // --- Address Form ---
            <ActionForm action=save_postal_address attr:data-testid="form-address">
                // Hidden meta fields the server expects (id / version / intent)
                <input
                    type="hidden"
                    name="id"
                    data-testid="hidden-id"
                    prop:value=id.unwrap_or(Uuid::nil()).to_string()
                />
                <input
                    type="hidden"
                    name="version"
                    data-testid="hidden-version"
                    prop:value=version
                />
                // --- Conflict Banner ---
                {move || {
                    if is_conflict() {
                        view! {
                            <AcknowledgmentBanner
                                msg="A newer version of this address exists. Reloading will discard your changes."
                                ack_btn_text="Reload"
                                ack_action=refetch_and_reset
                            />
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}

                // --- Duplicate Banner ---
                {move || {
                    if is_duplicate() {
                        view! {
                            <AcknowledgmentBanner
                                msg=format!(
                                    "An address with name '{}' already exists in '{} {}'. ",
                                    name.get(),
                                    postal_code.get(),
                                    locality.get(),
                                )
                                ack_btn_text="Ok"
                                ack_action=move || save_postal_address.clear()
                            />
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}
                // --- General Save Error Banner ---
                {move || {
                    if let Some(msg) = is_general_error() {
                        view! {
                            <AcknowledgmentAndNavigateBanner
                                msg=format!("An unexpected error occurred during saving: {msg}")
                                ack_btn_text="Dismiss"
                                ack_action=move || save_postal_address.clear()
                                nav_btn_text="Return to Search Address"
                                navigate_url=cancel_target()
                            />
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}

                // --- Address Form Fields ---
                // Disable the whole form while loading existing data or some conflict/error state
                <fieldset prop:disabled=is_disabled>
                    <label class="block">
                        <span class="block text-sm">"Name"</span>
                        <input
                            class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                            name="name"
                            data-testid="input-name"
                            aria-invalid=move || if is_valid_name() { "false" } else { "true" }
                            prop:value=name
                            placeholder=move || {
                                if addr_res.get().is_none() {
                                    "Loading..."
                                } else if is_new {
                                    "Enter name..."
                                } else {
                                    ""
                                }
                            }
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                            on:blur=move |_| {
                                set_name.set(current_address().get_name().to_string());
                            }
                        />
                    </label>
                    <label class="block">
                        <span class="block text-sm">"Street & number"</span>
                        <input
                            class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                            name="street"
                            data-testid="input-street"
                            aria-invalid=move || if is_valid_street() { "false" } else { "true" }
                            prop:value=street
                            placeholder=move || {
                                if addr_res.get().is_none() {
                                    "Loading..."
                                } else if is_new {
                                    "Enter street and number..."
                                } else {
                                    ""
                                }
                            }
                            on:input=move |ev| set_street.set(event_target_value(&ev))
                            on:blur=move |_| {
                                set_street.set(current_address().get_street().to_string());
                            }
                        />
                    </label>
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
                                prop:value=postal_code
                                placeholder=move || {
                                    if addr_res.get().is_none() {
                                        "Loading..."
                                    } else if is_new {
                                        "Enter postal code..."
                                    } else {
                                        ""
                                    }
                                }
                                on:input=move |ev| set_postal_code.set(event_target_value(&ev))
                                on:blur=move |_| {
                                    set_postal_code
                                        .set(current_address().get_postal_code().to_string());
                                }
                            />
                        </label>
                        <label class="block">
                            <span class="block text-sm">"City"</span>
                            <input
                                class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                                name="locality"
                                data-testid="input-locality"
                                aria-invalid=move || {
                                    if is_valid_locality() { "false" } else { "true" }
                                }
                                prop:value=locality
                                placeholder=move || {
                                    if addr_res.get().is_none() {
                                        "Loading..."
                                    } else if is_new {
                                        "Enter city..."
                                    } else {
                                        ""
                                    }
                                }
                                on:input=move |ev| set_locality.set(event_target_value(&ev))
                                on:blur=move |_| {
                                    set_locality.set(current_address().get_locality().to_string());
                                }
                            />
                        </label>
                    </div>
                    <label class="block">
                        <span class="block text-sm">"Region (optional)"</span>
                        <input
                            class="w-full border rounded p-2"
                            name="region"
                            data-testid="input-region"
                            prop:value=region
                            placeholder=move || {
                                if addr_res.get().is_none() {
                                    "Loading..."
                                } else if is_new {
                                    "Enter region (optional)..."
                                } else {
                                    ""
                                }
                            }
                            on:input=move |ev| set_region.set(event_target_value(&ev))
                            on:blur=move |_| {
                                set_region
                                    .set(
                                        current_address()
                                            .get_region()
                                            .unwrap_or_default()
                                            .to_string(),
                                    );
                            }
                        />
                    </label>
                    <label class="block">
                        <span class="block text-sm">"Country (ISO/name)"</span>
                        <input
                            class="w-full border rounded p-2 input aria-[invalid=true]:border-error aria-[invalid=true]:focus:outline-error"
                            name="country"
                            data-testid="input-country"
                            aria-invalid=move || if is_valid_country() { "false" } else { "true" }
                            prop:value=country
                            placeholder=move || {
                                if addr_res.get().is_none() {
                                    "Loading..."
                                } else if is_new {
                                    "Enter country..."
                                } else {
                                    ""
                                }
                            }
                            on:input=move |ev| set_country.set(event_target_value(&ev))
                            on:blur=move |_| {
                                set_country.set(current_address().get_country().to_string());
                            }
                        />
                    </label>
                    <div class="flex gap-2">
                        // Update existing
                        <button
                            type="submit"
                            name="intent"
                            value=move || if is_new { "create" } else { "update" }
                            data-testid="btn-save"
                            class="btn"
                            prop:disabled=move || is_disabled() || !is_valid_addr()
                        >
                            "Save"
                        </button>

                        // "save as new" (disabled in "new" mode)
                        <button
                            type="submit"
                            name="intent"
                            value="create"
                            data-testid="btn-save-as-new"
                            class="btn"
                            prop:disabled=move || is_disabled() || is_new || !is_valid_addr()
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
        </Transition>
    }
}
