// server function for postal address

use crate::{AppError, AppResult};
use app_core::{CoreState, PostalAddress, utils::id_version::IdVersion};
use leptos::prelude::*;
use tracing::{error, info, instrument};
use uuid::Uuid;

#[server]
#[instrument(
    name = "postal_address.load",
    skip_all,
    fields(id = %id)
)]
pub async fn load_postal_address(id: Uuid) -> AppResult<PostalAddress> {
    let mut core = expect_context::<CoreState>().as_postal_address_state();
    let pa = if let Some(pa) = core.load(id).await? {
        info!("loaded");
        pa.to_owned()
    } else {
        // Not an error here: returning default is the expected fallback
        info!("not_found_return_default");
        PostalAddress::default()
    };
    Ok(pa)
}

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
        name_len = name.as_deref().map(|s| s.len()).unwrap_or(0),
        locality_len = locality.len()
    )
)]
pub async fn save_postal_address(
    // hidden in the form; nil => new; else => update
    id: Uuid,
    // hidden in the form; -1 => new; else => update
    version: i64,
    // optional text field: treat "" as None
    name: Option<String>,
    street: String,
    postal_code: String,
    locality: String,
    // optional text field: treat "" as None
    region: Option<String>,
    country: String,
    // which submit button was clicked: "update" | "create"
    intent: Option<String>,
) -> AppResult<()> {
    let mut core = expect_context::<CoreState>().as_postal_address_state();

    // get mut handle to wrapped PostalAddress
    let mut_pa_core = core.get_mut();

    // Interpret intent
    let is_update = matches!(intent.as_deref(), Some("update"));
    if is_update {
        // set id and version previously loaded
        if version < 0 {
            return Err(AppError::NegativeVersionUpdate);
        }
        let id_version = IdVersion::builder()
            .set_id(id)
            .map_err(|_| AppError::NilIdUpdate)?
            .set_version(version as u64)
            .build();
        mut_pa_core.set_id_version(id_version);
        info!("saving_update");
    } else {
        info!("saving_create");
    }

    // set address data from Form inputs
    mut_pa_core
        .set_name(name.unwrap_or_default())
        .set_street(street)
        .set_postal_code(postal_code)
        .set_locality(locality)
        .set_region(region.unwrap_or_default())
        .set_country(country)
        .validate()?;

    // Persist; log outcome with the saved id.
    match core.save().await {
        Ok(saved) => {
            info!(saved_id = %saved.get_id(), "save_ok_redirect");
            let route = format!("/postal-address/{}", saved.get_id());
            leptos_axum::redirect(&route);
            Ok(())
        }
        Err(e) => {
            // Primary goal failed -> error
            error!(error = %e, "save_failed");
            Err(e.into())
        }
    }
}

#[server]
#[instrument(
    name = "postal_address.list",
    skip_all,
    fields(q_len = name.len(), limit = 10)
)]
pub async fn list_postal_addresses(name: String) -> AppResult<Vec<PostalAddress>> {
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
