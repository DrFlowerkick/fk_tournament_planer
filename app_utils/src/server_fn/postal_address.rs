//! Postal Address Server Functions Module

use crate::error::AppResult;
use app_core::PostalAddress;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::{
    CoreState,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use leptos::prelude::*;
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
pub async fn list_postal_address_ids(name: String, limit: Option<usize>) -> AppResult<Vec<Uuid>> {
    list_postal_address_ids_inner(name, limit).await
}

#[cfg(feature = "test-mock")]
pub async fn list_postal_address_ids(name: String, limit: Option<usize>) -> AppResult<Vec<Uuid>> {
    list_postal_address_ids_inner(name, limit).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn list_postal_address_ids_inner(
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<Uuid>> {
    let core = expect_context::<CoreState>().as_postal_address_state();
    info!("list_request");
    match core.list_address_ids(Some(&name), limit).await {
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

#[server]
#[instrument(
    name = "postal_address.save",
    skip_all,
    fields(
        id = %postal_address.get_id(),
        version = ?postal_address.get_version(),
        // tiny hints only; avoid PII/body dumps
        name_len = postal_address.get_name().len(),
        locality_len = postal_address.get_locality().len()
    )
)]
pub async fn save_postal_address(postal_address: PostalAddress) -> AppResult<PostalAddress> {
    save_postal_address_inner(postal_address).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn save_postal_address_inner(postal_address: PostalAddress) -> AppResult<PostalAddress> {
    let mut core = expect_context::<CoreState>().as_postal_address_state();

    // Interpret intent (create vs update) based on presence of id and version in the incoming postal address
    match postal_address.get_id_version() {
        IdVersion::Existing(..) => {
            info!("saving_update");
        }
        IdVersion::NewWithId(..) => {
            info!("saving_create");
        }
    }

    // We replace the state object in the core directly with the received object.
    // Prerequisite: The client has already set the correct IdVersion.
    *core.get_mut() = postal_address;

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
