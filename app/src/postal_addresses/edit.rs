use super::{
    AddressParams,
    server_fn::{SavePostalAddress, load_postal_address},
};
use crate::AppError;
use app_core::{PaValidationField, PostalAddress};
// this is probably in most cases only used for debugging. Remove it if not used anymore.
//use leptos::logging::log;
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
    // --- Server Actions & Resources ---
    let save_postal_address = ServerAction::<SavePostalAddress>::new();
    let addr_res = Resource::new(
        move || id,
        |maybe_id| async move {
            match maybe_id {
                Some(id) => load_postal_address(id).await,
                None => Ok(Default::default()),
            }
        },
    );

    let refetch = move || addr_res.refetch();

    // --- Signals for form fields ---
    let (name, set_name) = signal(String::new());
    let (street, set_street) = signal(String::new());
    let (postal_code, set_postal_code) = signal(String::new());
    let (locality, set_locality) = signal(String::new());
    let (region, set_region) = signal(String::new());
    let (country, set_country) = signal(String::new());
    let (version, set_version) = signal(0);

    // --- Signals for UI state & errors ---
    let (is_conflict, set_is_conflict) = signal(false);
    let (is_duplicate, set_is_duplicate) = signal(false);
    let (is_general_error, set_is_general_error) = signal(None::<String>);
    let pending = save_postal_address.pending();
    let action_value = save_postal_address.value();
    let navigate = use_navigate();

    let is_new = id.is_none();

    // --- Effect to populate signals from resource ---
    Effect::new(move |_| {
        addr_res
            .get()
            .and_then(|res| res.ok())
            .map(|addr| {
                // This runs when the resource loads or reloads.
                // It resets all form fields to the loaded data.
                set_name.set(addr.get_name().to_string());
                set_street.set(addr.get_street().to_string());
                set_postal_code.set(addr.get_postal_code().to_string());
                set_locality.set(addr.get_locality().to_string());
                set_region.set(addr.get_region().unwrap_or_default().to_string());
                set_country.set(addr.get_country().to_string());
                set_version.set(addr.get_version().unwrap_or_default());
                // Also reset error states on reload
                set_is_conflict.set(false);
                set_is_duplicate.set(false);
                set_is_general_error.set(None);
            });
    });

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

    // --- Effect to handle server action results (e.g., conflicts) ---
    Effect::new(move |_prev_val| {
        if let Some(result) = action_value.get() {
            match result {
                Err(AppError::Db(ref msg)) => {
                    if msg.contains("optimistic lock conflict") {
                        set_is_conflict.set(true);
                    } else if msg.contains("unique violation") {
                        set_is_duplicate.set(true);
                    } else {
                        set_is_general_error.set(Some(msg.clone()));
                    }
                }
                Err(ref e) => {
                    set_is_general_error.set(Some(format!("{:?}", e)));
                }
                Ok(_) => {
                    // On successful save, reset all error states
                    set_is_conflict.set(false);
                    set_is_duplicate.set(false);
                    set_is_general_error.set(None);
                }
            }
        }
    });

    let is_disabled = move || {
        addr_res.get().is_none()
            || pending.get()
            || is_conflict.get()
            || is_duplicate.get()
            || is_general_error.get().is_some()
    };

    let cancel_target = move || {
        id.map(|id| format!("/postal-address/{}", id))
            .unwrap_or_else(|| "/postal-address".to_string())
    };

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            // Use <ActionForm/> to bind to your save server fn
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
                    if is_conflict.get() {
                        view! { <ConflictBanner refetch=refetch /> }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                // --- Duplicate Banner ---
                {move || {
                    if is_duplicate.get() {
                        view! {
                            <DuplicateBanner
                                name=name.get()
                                postal_code=postal_code.get()
                                locality=locality.get()
                                set_is_duplicate=set_is_duplicate.clone()
                            />
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}
                // --- General Error Banner ---
                {move || {
                    if let Some(msg) = is_general_error.get() {
                        view! {
                            <GeneralErrorBanner
                                msg=msg
                                set_is_general_error=set_is_general_error.clone()
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

#[component]
fn ConflictBanner(refetch: impl Fn() + 'static) -> impl IntoView {
    view! {
        <div
            data-testid="conflict-banner"
            class="p-3 my-2 text-sm text-error-content bg-error rounded-lg"
            role="alert"
        >
            <p>"A newer version of this address exists. Reloading will discard your changes."</p>
            <button
                type="button"
                data-testid="btn-conflict-reload"
                class="btn btn-sm btn-outline mt-2"
                on:click=move |_| refetch()
            >
                "Reload"
            </button>
        </div>
    }
}

#[component]
fn DuplicateBanner(
    name: String,
    postal_code: String,
    locality: String,
    set_is_duplicate: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div
            data-testid="duplicate-banner"
            class="p-3 my-2 text-sm text-error-content bg-error rounded-lg"
            role="alert"
        >
            <p>
                {format!(
                    "An address with name '{name}' already exists in '{postal_code} {locality}'. ",
                )}
            </p>
            <button
                type="button"
                data-testid="btn-duplicate-dismiss"
                class="btn btn-sm btn-outline mt-2"
                on:click=move |_| set_is_duplicate.set(false)
            >
                "Reload"
            </button>
        </div>
    }
}

#[component]
fn GeneralErrorBanner(
    msg: String,
    set_is_general_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div
            class="p-3 my-2 text-sm text-error-content bg-error rounded-lg"
            role="alert"
            data-testid="generic-error-banner"
        >
            <p>{format!("An unexpected error occurred: {msg}")}</p>
            <button
                class="btn btn-sm btn-outline mt-2"
                data-testid="btn-generic-error-dismiss"
                on:click=move |_| set_is_general_error.set(None)
            >
                "OK"
            </button>
        </div>
    }
}
