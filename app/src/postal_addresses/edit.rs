//! Postal Address Edit Module

use app_core::PostalAddress;
use app_utils::{
    components::{
        banner::{AcknowledgmentAndNavigateBanner, AcknowledgmentBanner},
        inputs::{TextInput, ValidatedSelect, ValidatedTextInput},
    },
    error::AppError,
    state::global_state::{GlobalState, GlobalStateStoreFields},
    hooks::{
        is_field_valid::is_field_valid,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::AddressParams,
    server_fn::postal_address::{SavePostalAddress, load_postal_address},
};
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
    let cancel_target = Callback::new(move |_: ()| {
        format!("{}{}", return_after_address_edit.get(), query_string.get())
    });

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
                    Ok(None) => Err(AppError::ResourceNotFound("Postal Address".to_string(), id)),
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

    // --- Signals for UI state & errors ---
    // reset these signals with save_postal_address.clear() when needed
    let pending = save_postal_address.pending();

    let is_conflict = move || {
        if let Some(Err(AppError::Core(ce))) = save_postal_address.value().get()
            && ce.is_optimistic_lock_conflict()
        {
            true
        } else {
            false
        }
    };
    let is_duplicate = move || {
        if let Some(Err(AppError::Core(ce))) = save_postal_address.value().get()
            && ce.is_unique_violation()
        {
            true
        } else {
            false
        }
    };
    let is_general_error = move || {
        if let Some(Err(err)) = save_postal_address.value().get() {
            match err {
                AppError::Core(ce) => {
                    if ce.is_optimistic_lock_conflict() || ce.is_unique_violation() {
                        None
                    } else {
                        Some(format!("{:?}", ce))
                    }
                }
                _ => Some(format!("{:?}", err)),
            }
        } else {
            None
        }
    };
    let is_addr_res_error = move || matches!(addr_res.get(), Some(Err(_)));

    let is_disabled = move || {
        addr_res.get().is_none()
            || pending.get()
            || is_conflict()
            || is_duplicate()
            || is_addr_res_error()
            || is_general_error().is_some()
    };

    // --- Props for form fields ---
    let props = FormFieldsProperties {
        id,
        addr_res,
        cancel_target,
        is_disabled: Signal::derive(is_disabled),
        is_new: Signal::derive(is_new),
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
                        if is_new() { "New Postal Address" } else { "Edit Postal Address" }
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
                                            navigate_url=cancel_target.run(())
                                        />
                                    }
                                        .into_any()
                                }
                                Ok(_addr) => {
                                    view! {
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
                                        {move || {
                                            if let Some(msg) = is_general_error() {
                                                view! {
                                                    <AcknowledgmentAndNavigateBanner
                                                        msg=format!(
                                                            "An unexpected error occurred during saving: {msg}",
                                                        )
                                                        ack_btn_text="Dismiss"
                                                        ack_action=move || save_postal_address.clear()
                                                        nav_btn_text="Return to Search Address"
                                                        navigate_url=cancel_target.run(())
                                                    />
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                        // --- Address Form ---
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
#[derive(Clone, Copy)]
struct FormFieldsProperties {
    id: Signal<Option<Uuid>>,
    addr_res: Resource<Result<PostalAddress, AppError>>,
    cancel_target: Callback<(), String>,
    is_disabled: Signal<bool>,
    is_new: Signal<bool>,
    set_name: RwSignal<String>,
    set_street: RwSignal<String>,
    set_postal_code: RwSignal<String>,
    set_locality: RwSignal<String>,
    set_region: RwSignal<String>,
    set_country: RwSignal<String>,
    set_version: RwSignal<u32>,
}

#[component]
fn FormFields(props: FormFieldsProperties) -> impl IntoView {
    let FormFieldsProperties {
        id,
        addr_res,
        cancel_target,
        is_disabled,
        is_new,
        set_name,
        set_street,
        set_postal_code,
        set_locality,
        set_region,
        set_country,
        set_version,
    } = props;
    let navigate = use_navigate();

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

    let is_loading = Signal::derive(move || addr_res.get().is_none());

    let validation_result = move || current_address().validate();
    let is_valid_addr = move || validation_result().is_ok();

    let is_valid_name = Signal::derive(move || is_field_valid(validation_result).run("Name"));
    let is_valid_street = Signal::derive(move || is_field_valid(validation_result).run("Street"));
    let is_valid_postal_code =
        Signal::derive(move || is_field_valid(validation_result).run("PostalCode"));
    let is_valid_locality =
        Signal::derive(move || is_field_valid(validation_result).run("Locality"));
    let is_valid_country = Signal::derive(move || is_field_valid(validation_result).run("Country"));
    let countries = get_sorted_countries();

    view! {
        // --- Address Form Fields ---
        <fieldset class="space-y-4" prop:disabled=is_disabled>
            // Hidden meta fields the server expects (id / version)
            <input
                type="hidden"
                name="id"
                data-testid="hidden-id"
                prop:value=move || id.get().unwrap_or(Uuid::nil()).to_string()
            />
            <input
                type="hidden"
                name="version"
                data-testid="hidden-version"
                prop:value=set_version
            />
            <ValidatedTextInput
                label="Name"
                name="name"
                value=set_name
                error_message=is_valid_name
                is_loading=is_loading
                is_new=is_new
                on_blur=move || set_name.set(current_address().get_name().to_string())
            />
            <ValidatedTextInput
                label="Street & number"
                name="street"
                value=set_street
                error_message=is_valid_street
                is_loading=is_loading
                is_new=is_new
                on_blur=move || set_street.set(current_address().get_street().to_string())
            />
            <div class="grid grid-cols-2 gap-4">
                <ValidatedTextInput
                    label="Postal code"
                    name="postal_code"
                    value=set_postal_code
                    error_message=is_valid_postal_code
                    is_loading=is_loading
                    is_new=is_new
                    on_blur=move || {
                        set_postal_code.set(current_address().get_postal_code().to_string())
                    }
                />
                <ValidatedTextInput
                    label="City"
                    name="locality"
                    value=set_locality
                    error_message=is_valid_locality
                    is_loading=is_loading
                    is_new=is_new
                    on_blur=move || set_locality.set(current_address().get_locality().to_string())
                />
            </div>
            <TextInput
                label="Region"
                name="region"
                value=set_region
                optional=true
                is_loading=is_loading
                is_new=is_new
                on_blur=move || {
                    set_region.set(current_address().get_region().unwrap_or_default().to_string())
                }
            />
            <ValidatedSelect
                label="Country"
                name="country"
                value=set_country
                error_message=is_valid_country
                options=countries
            />
            <div class="card-actions justify-end mt-4">
                <button
                    type="submit"
                    name="intent"
                    value=move || if is_new.get() { "create" } else { "update" }
                    data-testid="btn-save"
                    class="btn btn-primary"
                    prop:disabled=move || is_disabled.get() || !is_valid_addr()
                >
                    "Save"
                </button>

                <button
                    type="submit"
                    name="intent"
                    value="create"
                    data-testid="btn-save-as-new"
                    class="btn btn-secondary"
                    prop:disabled=move || is_disabled.get() || is_new.get() || !is_valid_addr()
                    prop:hidden=move || is_new.get()
                >
                    "Save as new"
                </button>

                <button
                    type="button"
                    name="intent"
                    value="cancel"
                    data-testid="btn-cancel"
                    class="btn btn-ghost"
                    on:click=move |_| navigate(&cancel_target.run(()), NavigateOptions::default())
                >
                    "Cancel"
                </button>
            </div>
        </fieldset>
    }
}
