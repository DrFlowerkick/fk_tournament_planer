//! Postal Address Server Functions Module

#[cfg(feature = "test-mock")]
pub mod test_support;

// server function for postal address

#[cfg(any(feature = "ssr", feature = "test-mock"))]
use crate::error::AppError;
use crate::error::AppResult;
use app_core::PostalAddress;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::{CoreState, utils::id_version::IdVersion};
use leptos::prelude::*;
#[cfg(not(feature = "test-mock"))]
use tracing::instrument;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use tracing::{error, info};
use uuid::Uuid;

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "postal_address.load",
    skip_all,
    fields(id = %id)
)]
pub async fn load_postal_address(id: Uuid) -> AppResult<Option<PostalAddress>> {
    load_postal_address_inner(id).await
}

#[cfg(feature = "test-mock")]
pub async fn load_postal_address(id: Uuid) -> AppResult<Option<PostalAddress>> {
    load_postal_address_inner(id).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn load_postal_address_inner(id: Uuid) -> AppResult<Option<PostalAddress>> {
    let mut core = expect_context::<CoreState>().as_postal_address_state();
    let pa = core.load(id).await?.map(|pa| pa.to_owned());
    Ok(pa)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "postal_address.list",
    skip_all,
    fields(q_len = name.len(), limit = 10)
)]
pub async fn list_postal_addresses(name: String) -> AppResult<Vec<PostalAddress>> {
    list_postal_addresses_inner(name).await
}

#[cfg(feature = "test-mock")]
pub async fn list_postal_addresses(name: String) -> AppResult<Vec<PostalAddress>> {
    list_postal_addresses_inner(name).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn list_postal_addresses_inner(name: String) -> AppResult<Vec<PostalAddress>> {
    let core = expect_context::<CoreState>().as_postal_address_state();
    info!("list_request");
    match core.list_addresses(Some(&name), Some(10)).await {
        Ok(list) => {
            info!(count = list.len(), "list_ok");
            Ok(list)
        }
        Err(e) => {
            error!(error = %e, "list_failed");
            Err(e.into())
        }
    }
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "postal_address.save",
    skip_all,
    fields(
        id = %id,
        version = version,
        // capture intent without logging full payloads
        intent = intent.as_deref().unwrap_or(""),
        // tiny hints only; avoid PII/body dumps
        name_len = name.len(),
        locality_len = locality.len()
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn save_postal_address(
    // hidden in the form; nil => new; else => update
    id: Uuid,
    // hidden in the form
    version: u32,
    name: String,
    street: String,
    postal_code: String,
    locality: String,
    // optional text field: treat "" as None
    region: Option<String>,
    country: String,
    // which submit button was clicked: "update" | "create"
    intent: Option<String>,
) -> AppResult<PostalAddress> {
    save_postal_address_inner(
        id,
        version,
        name,
        street,
        postal_code,
        locality,
        region,
        country,
        intent,
    )
    .await
}

#[cfg(feature = "test-mock")]
pub use test_support::SavePostalAddress;

#[cfg(feature = "test-mock")]
#[allow(clippy::too_many_arguments)]
pub async fn save_postal_address(
    id: Uuid,
    version: u32,
    name: String,
    street: String,
    postal_code: String,
    locality: String,
    region: Option<String>,
    country: String,
    intent: Option<String>,
) -> AppResult<PostalAddress> {
    save_postal_address_inner(
        id,
        version,
        name,
        street,
        postal_code,
        locality,
        region,
        country,
        intent,
    )
    .await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
#[allow(clippy::too_many_arguments)]
pub async fn save_postal_address_inner(
    id: Uuid,
    version: u32,
    name: String,
    street: String,
    postal_code: String,
    locality: String,
    region: Option<String>,
    country: String,
    intent: Option<String>,
) -> AppResult<PostalAddress> {
    let mut core = expect_context::<CoreState>().as_postal_address_state();

    // get mut handle to wrapped PostalAddress
    let mut_pa_core = core.get_mut();

    // Interpret intent
    let is_update = matches!(intent.as_deref(), Some("update"));
    if is_update {
        // set id and version previously loaded
        if id.is_nil() {
            return Err(AppError::NilIdUpdate);
        }
        let id_version = IdVersion::new(id, Some(version));
        mut_pa_core.set_id_version(id_version);
        info!("saving_update");
    } else {
        info!("saving_create");
    }

    // set address data from Form inputs
    mut_pa_core
        .set_name(name)
        .set_street(street)
        .set_postal_code(postal_code)
        .set_locality(locality)
        .set_region(region.unwrap_or_default())
        .set_country(country);

    // Persist; log outcome with the saved id. if save() is ok, it returns valid id -> unwrap() is save
    match core.save().await {
        Ok(saved) => {
            info!(saved_id = %saved.get_id(), "save_ok");
            Ok(saved.clone())
        }
        Err(e) => {
            // Primary goal failed -> error
            error!(error = %e, "save_failed");
            Err(e.into())
        }
    }
}
