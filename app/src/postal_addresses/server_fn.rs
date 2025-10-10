// server function for postal address

use leptos::prelude::*;
use crate::AppResult;
use app_core::{PostalAddress, CoreState};
use uuid::Uuid;

#[server]
pub async fn load_postal_address(id: Uuid) -> AppResult<PostalAddress> {
    let mut core = expect_context::<CoreState>().as_postal_address_state();
    let pa = if let Some(pa) = core.load(id).await? {
        pa.to_owned()
    } else {
        PostalAddress::default()
    };
    Ok(pa)
}

#[server]
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
    let mut core = expect_context::<CoreState>().as_postal_address_state();

    if matches!(intent.as_deref(), Some("update")) {
        // set id and version previously loaded
        core.set_id(id);
        core.set_version(version);
    }

    let name = name.unwrap_or_default();
    core.change_name(name);
    core.change_street_address(street_address);
    core.change_postal_code(postal_code);
    core.change_address_locality(address_locality);
    let address_region = address_region.unwrap_or_default();
    core.change_address_region(address_region);
    core.change_address_country(address_country);

    // ToDo: gracefully handle errors, e.g. retry
    let saved = core.save().await?;
    let route = format!("/postal-address/{}", saved.id);
    // redirect to newly saved postal address
    leptos_axum::redirect(&route);
    Ok(())
}

#[server]
pub async fn list_postal_addresses(name: String) -> AppResult<Vec<PostalAddress>> {
    let core = expect_context::<CoreState>().as_postal_address_state();
    let list = core.list_addresses(Some(&name), Some(10)).await?;
    Ok(list)
}
