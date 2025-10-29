// server function for postal address

use crate::AppResult;
use app_core::PostalAddress;
use leptos::prelude::*;
use tracing::instrument;
#[cfg(feature = "ssr")]
use tracing::info;
use uuid::Uuid;

#[server]
#[instrument(
    name = "postal_address.load",
    skip_all,
    fields(id = %id)
)]
pub async fn load_postal_address_dummy(id: Uuid) -> AppResult<Option<PostalAddress>> {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    info!("load_dummy_called");
    Ok(Some(Default::default()))
}
