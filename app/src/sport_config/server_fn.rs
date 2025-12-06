//! Server functions for sport configuration.

use crate::error::AppResult;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::CoreState;
use app_core::SportConfig;
use leptos::prelude::*;
#[cfg(not(feature = "test-mock"))]
use tracing::instrument;
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
#[instrument(name = "sport_config.list_sport_configs", skip_all)]
pub async fn list_sport_configs(
    sport_id: Uuid,
    name: String,
) -> AppResult<Vec<app_core::SportConfig>> {
    list_sport_configs_inner(sport_id, name).await
}

#[cfg(feature = "test-mock")]
pub async fn list_sport_configs(
    sport_id: Uuid,
    name: String,
) -> AppResult<Vec<app_core::SportConfig>> {
    list_sport_configs_inner(sport_id, name).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
async fn list_sport_configs_inner(
    sport_id: Uuid,
    name: String,
) -> AppResult<Vec<app_core::SportConfig>> {
    let core = expect_context::<CoreState>().as_sport_config_state();
    let configs = core
        .list_sport_configs(sport_id, Some(&name), Some(10))
        .await?;
    Ok(configs)
}
