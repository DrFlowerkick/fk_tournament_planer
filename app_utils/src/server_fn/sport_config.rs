//! Sport Config Server Functions Module

use crate::error::AppResult;
use app_core::SportConfig;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::{
    CoreState,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use leptos::{prelude::*, server_fn::codec::Json};
use tracing::instrument;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use tracing::{error, info};
use uuid::Uuid;

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(
    name = "sport_config.load",
    skip_all,
    fields(id = %id)
)]
pub async fn load_sport_config(id: Uuid) -> AppResult<Option<SportConfig>> {
    load_sport_config_inner(id).await
}

#[cfg(feature = "test-mock")]
pub async fn load_sport_config(id: Uuid) -> AppResult<Option<SportConfig>> {
    load_sport_config_inner(id).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn load_sport_config_inner(id: Uuid) -> AppResult<Option<SportConfig>> {
    let mut core = expect_context::<CoreState>().as_sport_config_state();
    let sc = core.load(id).await?.map(|sc| sc.to_owned());
    Ok(sc)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(name = "sport_config.list_sport_config_ids", skip_all)]
pub async fn list_sport_config_ids(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<Uuid>> {
    list_sport_configs_inner(sport_id, name, limit).await
}

#[cfg(feature = "test-mock")]
pub async fn list_sport_config_ids(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<Uuid>> {
    list_sport_configs_inner(sport_id, name, limit).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
async fn list_sport_configs_inner(
    sport_id: Uuid,
    name: String,
    limit: Option<usize>,
) -> AppResult<Vec<Uuid>> {
    let core = expect_context::<CoreState>().as_sport_config_state();
    let configs = core
        .list_sport_config_ids(sport_id, Some(&name), limit)
        .await?;
    Ok(configs)
}

#[server(input = Json, output = Json)]
#[instrument(
    name = "sport_config.save",
    skip_all,
    fields(
        id = %sport_config.get_id(),
        version = ?sport_config.get_version(),
        name_len = sport_config.get_name().len(),
    )
)]
pub async fn save_sport_config(sport_config: SportConfig) -> AppResult<SportConfig> {
    save_sport_config_inner(sport_config).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
pub async fn save_sport_config_inner(sport_config: SportConfig) -> AppResult<SportConfig> {
    let mut core = expect_context::<CoreState>().as_sport_config_state();

    // Interpret intent from presence of id and version, as sent by the client in the form data
    match sport_config.get_id_version() {
        IdVersion::Existing(..) => {
            info!("saving_update");
        }
        IdVersion::NewWithId(..) => {
            info!("saving_create");
        }
    }

    // We replace the state object in the core directly with the received object.
    // Prerequisite: The client has already set the correct IdVersion.
    *core.get_mut() = sport_config;

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
