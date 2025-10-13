// server function for postal address

use crate::AppResult;
use app_core::{CoreServerState, PostalAddress};
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
    let mut core = expect_context::<CoreServerState>().as_postal_address_state();
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
        locality_len = address_locality.len()
    )
)]
pub async fn save_postal_address(
    // hidden in the form; nil => new; else => update
    id: Uuid,
    // hidden in the form; -1 => new; else => update
    version: i64,
    // optional text field: treat "" as None
    name: Option<String>,
    street_address: String,
    postal_code: String,
    address_locality: String,
    // optional text field: treat "" as None
    address_region: Option<String>,
    address_country: String,
    // which submit button was clicked: "update" | "create"
    intent: Option<String>,
) -> AppResult<()> {
    let mut core = expect_context::<CoreServerState>().as_postal_address_state();

    // Interpret intent
    let is_update = matches!(intent.as_deref(), Some("update"));
    if is_update {
        // set id and version previously loaded
        core.set_id(id);
        core.set_version(version);
        info!("saving_update");
    } else {
        info!("saving_create");
    }

    let name = name.unwrap_or_default();
    core.change_name(name);
    core.change_street_address(street_address);
    core.change_postal_code(postal_code);
    core.change_address_locality(address_locality);
    let address_region = address_region.unwrap_or_default();
    core.change_address_region(address_region);
    core.change_address_country(address_country);

    // Persist; log outcome with the saved id.
    match core.save().await {
        Ok(saved) => {
            info!(saved_id = %saved.id, "save_ok_redirect");
            let route = format!("/postal-address/{}", saved.id);
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
    let core = expect_context::<CoreServerState>().as_postal_address_state();
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
