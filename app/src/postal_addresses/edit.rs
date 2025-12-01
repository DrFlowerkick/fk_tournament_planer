use super::{
    AddressParams,
    server_fn::{SavePostalAddress, load_postal_address},
};
use crate::{
    AppError,
    components::banner::{AcknowledgmentAndNavigateBanner, AcknowledgmentBanner},
    global_state::{GlobalState, GlobalStateStoreFields},
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
};
use app_core::{PaValidationField, PostalAddress};
use isocountry::CountryCode;
use leptos::prelude::*;
#[cfg(feature = "test-mock")]
use leptos::{wasm_bindgen::JsCast, web_sys};
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
};
use reactive_stores::Store;
use uuid::Uuid;

fn get_sorted_countries() -> Vec<(String, String)> {
    let mut countries: Vec<(String, String)> = CountryCode::iter()
        .map(|c| (c.alpha2().to_string(), c.name().to_string()))
        .collect();
    // sort by country name
    countries.sort_by(|a, b| a.1.cmp(&b.1));
    countries
}

// Wrapper component to provide type safe refetch function via context
#[component]
pub fn PostalAddressForm() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        update,
        path,
        query_string,
        ..
    } = use_query_navigation();

    let is_new = move || path.read().ends_with("/new_pa") || path.read().is_empty();
    let query = use_query::<AddressParams>();
    let id = Signal::derive(move || {
        if is_new() {
            None
        } else {
            query.get().map(|ap| ap.address_id).unwrap_or(None)
        }
    });

    let state = expect_context::<Store<GlobalState>>();
    let return_after_address_edit = state.return_after_address_edit();
    let cancel_target =
        move || format!("{}{}", return_after_address_edit.get(), query_string.get());

    // --- Signals for form fields ---
    let set_name = RwSignal::new(String::new());
    let set_street = RwSignal::new(String::new());
    let set_postal_code = RwSignal::new(String::new());
    let set_locality = RwSignal::new(String::new());
    let set_region = RwSignal::new(String::new());
    let set_country = RwSignal::new(String::new());
    let set_version = RwSignal::new(0);

    // --- Server Actions & Resources ---
    let save_postal_address = ServerAction::<SavePostalAddress>::new();

    Effect::new(move || {
        if let Some(Ok(pa)) = save_postal_address.value().get() {
            save_postal_address.clear();
            update(
                "address_id",
                &pa.get_id().map(|id| id.to_string()).unwrap_or_default(),
            );
            let nav_url = format!("{}{}", return_after_address_edit.get(), query_string.get());
            let navigate = use_navigate();
            navigate(&nav_url, NavigateOptions::default());
        }
    });

    let addr_res = Resource::new(
        move || id.get(),
        move |maybe_id| async move {
            match maybe_id {
                Some(id) => match load_postal_address(id).await {
                    Ok(Some(addr)) => {
                        set_name.set(addr.get_name().to_string());
                        set_street.set(addr.get_street().to_string());
                        set_postal_code.set(addr.get_postal_code().to_string());
                        set_locality.set(addr.get_locality().to_string());
                        set_region.set(addr.get_region().unwrap_or_default().to_string());
                        set_country.set(addr.get_country().to_string());
                        set_version.set(addr.get_version().unwrap_or_default());
                        Ok(addr)
                    }
                    Ok(None) => Err(AppError::Db("Not found".to_string())),
                    Err(e) => Err(e),
                },
                None => Ok(Default::default()),
            }
        },
    );

    let refetch_and_reset = move || {
        save_postal_address.clear();
        addr_res.refetch();
    };

    // --- Props for form fields ---
    let props = FormFieldsProperties {
        id,
        save_postal_address,
        addr_res,
        refetch_and_reset,
        cancel_target,
        set_name,
        set_street,
        set_postal_code,
        set_locality,
        set_region,
        set_country,
        set_version,
    };

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">
                    {move || {
                        if is_new() {
                            "New Postal Address"
                        } else {
                            "Edit Postal Address"
                        }
                    }}
                </h2>
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
                                            ack_action=refetch_and_reset
                                            nav_btn_text="Cancel"
                                            navigate_url=cancel_target()
                                        />
                                    }
                                        .into_any()
                                }
                                Ok(_addr) => {
                                    let props = props.clone();
                                    view! {
                                        <div data-testid="form-address">
                                            {
                                                #[cfg(not(feature = "test-mock"))]
                                                {
                                                    view! {
                                                        <ActionForm action=save_postal_address>
                                                            <FormFields props=props />
                                                        </ActionForm>
                                                    }
                                                }
                                                #[cfg(feature = "test-mock")]
                                                {
                                                    view! {
                                                        <form on:submit=move |ev| {
                                                            ev.prevent_default();
                                                            let intent = ev
                                                                .submitter()
                                                                .and_then(|el| {
                                                                    el.dyn_into::<web_sys::HtmlButtonElement>().ok()
                                                                })
                                                                .map(|btn| btn.value());
                                                            let data = SavePostalAddress {
                                                                id: id.get().unwrap_or(Uuid::nil()),
                                                                version: set_version.get(),
                                                                name: set_name.get(),
                                                                street: set_street.get(),
                                                                postal_code: set_postal_code.get(),
                                                                locality: set_locality.get(),
                                                                region: Some(set_region.get()).filter(|r| !r.is_empty()),
                                                                country: set_country.get(),
                                                                intent,
                                                            };
                                                            save_postal_address.dispatch(data);
                                                        }>
                                                            <FormFields props=props />
                                                        </form>
                                                    }
                                                }
                                            }
                                        </div>
                                    }
                                        .into_any()
                                }
                            })
                    }}

                </Transition>
            </div>
        </div>
    }
}

// Props for form fields component
#[derive(Clone)]
struct FormFieldsProperties<RR: Fn(), CT: Fn() -> String> {
    id: Signal<Option<Uuid>>,
    save_postal_address: ServerAction<SavePostalAddress>,
    addr_res: Resource<Result<PostalAddress, AppError>>,
    refetch_and_reset: RR,
    cancel_target: CT,
    set_name: RwSignal<String>,
    set_street: RwSignal<String>,
    set_postal_code: RwSignal<String>,
    set_locality: RwSignal<String>,
    set_region: RwSignal<String>,
    set_country: RwSignal<String>,
    set_version: RwSignal<u32>,
}

#[component]
fn FormFields<RR: Fn() + Clone + Send + 'static, CT: Fn() -> String + Clone + Send + 'static>(
    props: FormFieldsProperties<RR, CT>,
) -> impl IntoView {
    let FormFieldsProperties {
        id,
        save_postal_address,
        addr_res,
        refetch_and_reset,
        cancel_target,
        set_name,
        set_street,
        set_postal_code,
        set_locality,
        set_region,
        set_country,
        set_version,
    } = props;
    let navigate = use_navigate();

    // --- Signals for UI state & errors ---
    let pending = save_postal_address.pending();
    let is_new = move || id.get().is_none();

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
        addr.set_name(set_name.get());
        addr.set_street(set_street.get());
        addr.set_postal_code(set_postal_code.get());
        addr.set_locality(set_locality.get());
        addr.set_country(set_country.get());
        if !set_region.get().is_empty() {
            addr.set_region(set_region.get());
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

    let countries = get_sorted_countries();

    view! {
        // Hidden meta fields the server expects (id / version / intent)
        <input
            type="hidden"
            name="id"
            data-testid="hidden-id"
            prop:value=move || id.get().unwrap_or(Uuid::nil()).to_string()
        />
        <input type="hidden" name="version" data-testid="hidden-version" prop:value=set_version />
        // --- Conflict Banner ---
        {move || {
            if is_conflict() {
                view! {
                    <AcknowledgmentBanner
                        msg="A newer version of this address exists. Reloading will discard your changes."
                        ack_btn_text="Reload"
                        ack_action=refetch_and_reset.clone()
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
                            set_name.get(),
                            set_postal_code.get(),
                            set_locality.get(),
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
        {
            let cancel_target = cancel_target.clone();
            move || {
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
            }
        }

        // --- Address Form Fields ---
        <fieldset class="space-y-4" prop:disabled=is_disabled>
            <div class="form-control w-full">
                <label class="label">
                    <span class="label-text">"Name"</span>
                </label>
                <input
                    type="text"
                    class="input input-bordered w-full"
                    class:input-error=move || !is_valid_name()
                    name="name"
                    data-testid="input-name"
                    aria-invalid=move || if is_valid_name() { "false" } else { "true" }
                    prop:value=set_name
                    placeholder=move || {
                        if addr_res.get().is_none() {
                            "Loading..."
                        } else if is_new() {
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
            </div>
            <div class="form-control w-full">
                <label class="label">
                    <span class="label-text">"Street & number"</span>
                </label>
                <input
                    type="text"
                    class="input input-bordered w-full"
                    class:input-error=move || !is_valid_street()
                    name="street"
                    data-testid="input-street"
                    aria-invalid=move || if is_valid_street() { "false" } else { "true" }
                    prop:value=set_street
                    placeholder=move || {
                        if addr_res.get().is_none() {
                            "Loading..."
                        } else if is_new() {
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
            </div>
            <div class="grid grid-cols-2 gap-4">
                <div class="form-control w-full">
                    <label class="label">
                        <span class="label-text">"Postal code"</span>
                    </label>
                    <input
                        type="text"
                        class="input input-bordered w-full"
                        class:input-error=move || !is_valid_postal_code()
                        name="postal_code"
                        data-testid="input-postal_code"
                        aria-invalid=move || {
                            if is_valid_postal_code() { "false" } else { "true" }
                        }
                        prop:value=set_postal_code
                        placeholder=move || {
                            if addr_res.get().is_none() {
                                "Loading..."
                            } else if is_new() {
                                "Enter postal code..."
                            } else {
                                ""
                            }
                        }
                        on:input=move |ev| set_postal_code.set(event_target_value(&ev))
                        on:blur=move |_| {
                            set_postal_code.set(current_address().get_postal_code().to_string());
                        }
                    />
                </div>
                <div class="form-control w-full">
                    <label class="label">
                        <span class="label-text">"City"</span>
                    </label>
                    <input
                        type="text"
                        class="input input-bordered w-full"
                        class:input-error=move || !is_valid_locality()
                        name="locality"
                        data-testid="input-locality"
                        aria-invalid=move || { if is_valid_locality() { "false" } else { "true" } }
                        prop:value=set_locality
                        placeholder=move || {
                            if addr_res.get().is_none() {
                                "Loading..."
                            } else if is_new() {
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
                </div>
            </div>
            <div class="form-control w-full">
                <label class="label">
                    <span class="label-text">"Region (optional)"</span>
                </label>
                <input
                    type="text"
                    class="input input-bordered w-full"
                    name="region"
                    data-testid="input-region"
                    prop:value=set_region
                    placeholder=move || {
                        if addr_res.get().is_none() {
                            "Loading..."
                        } else if is_new() {
                            "Enter region (optional)..."
                        } else {
                            ""
                        }
                    }
                    on:input=move |ev| set_region.set(event_target_value(&ev))
                    on:blur=move |_| {
                        set_region
                            .set(current_address().get_region().unwrap_or_default().to_string());
                    }
                />
            </div>
            <div class="form-control w-full">
                <label class="label">
                    <span class="label-text">"Country"</span>
                </label>
                <select
                    class="select select-bordered w-full"
                    class:select-error=move || !is_valid_country()
                    name="country"
                    data-testid="input-country"
                    aria-invalid=move || if is_valid_country() { "false" } else { "true" }
                    on:change=move |ev| set_country.set(event_target_value(&ev))
                    prop:value=set_country
                >
                    <option disabled selected value="">
                        "Select a country..."
                    </option>
                    {countries
                        .into_iter()
                        .map(|(code, name)| {
                            view! {
                                <option
                                    value=code.clone()
                                    selected=move || set_country.get() == code
                                >
                                    {name}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </div>
            <div class="card-actions justify-end mt-4">
                <button
                    type="submit"
                    name="intent"
                    value=move || if is_new() { "create" } else { "update" }
                    data-testid="btn-save"
                    class="btn btn-primary"
                    prop:disabled=move || is_disabled() || !is_valid_addr()
                >
                    "Save"
                </button>

                <button
                    type="submit"
                    name="intent"
                    value="create"
                    data-testid="btn-save-as-new"
                    class="btn btn-secondary"
                    prop:disabled=move || is_disabled() || is_new() || !is_valid_addr()
                    prop:hidden=move || is_new
                >
                    "Save as new"
                </button>

                <button
                    type="button"
                    name="intent"
                    value="cancel"
                    data-testid="btn-cancel"
                    class="btn btn-ghost"
                    on:click=move |_| navigate(&cancel_target.clone()(), NavigateOptions::default())
                >
                    "Cancel"
                </button>
            </div>
        </fieldset>
    }
}
