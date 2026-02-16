//! Postal Address Server Functions Module

#[cfg(any(feature = "ssr", feature = "test-mock"))]
use crate::error::AppError;
use crate::error::AppResult;
use app_core::PostalAddress;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::{CoreState, utils::id_version::IdVersion};
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use isocountry::CountryCode;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
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
    fields(q_len = name.len(), limit = limit.unwrap_or(10))
)]
pub async fn list_postal_addresses(
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<PostalAddress>> {
    list_postal_addresses_inner(name, limit).await
}

#[cfg(feature = "test-mock")]
pub async fn list_postal_addresses(
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<PostalAddress>> {
    list_postal_addresses_inner(name, limit).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn list_postal_addresses_inner(
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<PostalAddress>> {
    let core = expect_context::<CoreState>().as_postal_address_state();
    info!("list_request");
    match core.list_addresses(Some(&name), limit).await {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePostalAddressFormData {
    pub id: Uuid,
    pub version: u32,
    pub name: String,
    pub street: String,
    pub postal_code: String,
    pub locality: String,
    pub region: Option<String>,
    pub country: String,
    pub intent: Option<String>,
}

#[server]
#[instrument(
    name = "postal_address.save",
    skip_all,
    fields(
        id = %form.id,
        version = form.version,
        // capture intent without logging full payloads
        intent = form.intent.as_deref().unwrap_or(""),
        // tiny hints only; avoid PII/body dumps
        name_len = form.name.len(),
        locality_len = form.locality.len()
    )
)]
pub async fn save_postal_address(form: SavePostalAddressFormData) -> AppResult<PostalAddress> {
    save_postal_address_inner(form).await
}

/*
Replace by on:submit handler for test mock, which is at the moment defined at EditPostalAddress

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
}*/

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn save_postal_address_inner(
    form: SavePostalAddressFormData,
) -> AppResult<PostalAddress> {
    use app_core::{CoreError, DbError};

    let mut core = expect_context::<CoreState>().as_postal_address_state();

    // get mut handle to wrapped PostalAddress
    let mut_pa_core = core.get_mut();

    // Interpret intent
    // ToDo: we have to refactor this when switching to auto save.
    match form.intent.as_deref() {
        Some("update") => {
            // set id and version previously loaded
            if form.id.is_nil() {
                return Err(AppError::NilIdUpdate);
            }
            let id_version = IdVersion::new(form.id, Some(form.version));
            mut_pa_core.set_id_version(id_version);
            info!("saving_update");
        }
        Some("create") => {
            let id_version = IdVersion::new(form.id, None);
            mut_pa_core.set_id_version(id_version);
            info!("saving_create");
        }
        _ => { /* ToDo: should we return err for unknown intent? Or how do we handle this case? */ }
    }

    let country_code =
        CountryCode::for_alpha2(&form.country).map_err(|e| CoreError::from(DbError::from(e)))?;

    // set address data from Form inputs
    mut_pa_core
        .set_name(form.name)
        .set_street(form.street)
        .set_postal_code(form.postal_code)
        .set_locality(form.locality)
        .set_region(form.region.unwrap_or_default())
        .set_country(Some(country_code));

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
