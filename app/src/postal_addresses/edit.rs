use super::{AddressParams, server_fn::load_postal_address_dummy};
use crate::{AppError, banner::AcknowledgmentAndNavigateBanner};
use leptos::prelude::*;
use leptos_router::hooks::use_params;
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
    let addr_res = Resource::new(
        move || id,
        |maybe_id| async move {
            match maybe_id {
                Some(id) => match load_postal_address_dummy(id).await {
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
    };

    // --- Signals for form fields ---
    let (name, set_name) = signal(String::new());
    let (street, set_street) = signal(String::new());
    let (postal_code, set_postal_code) = signal(String::new());
    let (locality, set_locality) = signal(String::new());
    let (region, set_region) = signal(String::new());
    let (country, set_country) = signal(String::new());
    let (version, set_version) = signal(0);

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
            }} <p>"Postal Address Edit (debug version)"</p> <p>{name}</p> <p>{street}</p>
            <p>{postal_code}</p> <p>{locality}</p> <p>{region}</p> <p>{country}</p>
            <p>{move || format!("{}", id.unwrap_or_default().to_string())}</p> <p>{version}</p>
        </Transition>
    }
}
