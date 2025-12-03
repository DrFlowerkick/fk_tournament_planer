//! Server functions for sport configuration.

use crate::error::AppResult;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::CoreState;
use app_core::SportPluginInfo;
use leptos::prelude::*;
#[cfg(not(feature = "test-mock"))]
use tracing::instrument;
use uuid::Uuid;

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(name = "sport_config.list_sport_plugins", skip_all)]
pub async fn list_sport_plugins() -> AppResult<Vec<SportPluginInfo>> {
    list_sport_plugins_inner().await
}

#[cfg(feature = "test-mock")]
pub async fn list_sport_plugins() -> AppResult<Vec<SportPluginInfo>> {
    list_sport_plugins_inner().await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
async fn list_sport_plugins_inner() -> AppResult<Vec<SportPluginInfo>> {
    let core = expect_context::<CoreState>().as_sport_plugin_state();
    let plugins = core
        .list_sports()
        .iter()
        .map(SportPluginInfo::from)
        .collect();
    Ok(plugins)
}

#[cfg(not(feature = "test-mock"))]
#[server]
#[instrument(name = "sport_config.list_sport_configs", skip_all)]
pub async fn list_sport_configs(
    sport_id: Uuid,
    name_filter: Option<String>,
    limit: Option<usize>,
) -> AppResult<Vec<app_core::SportConfig>> {
    list_sport_configs_inner(sport_id, name_filter, limit).await
}

#[cfg(feature = "test-mock")]
pub async fn list_sport_configs(
    sport_id: Uuid,
    name_filter: Option<String>,
    limit: Option<usize>,
) -> AppResult<Vec<app_core::SportConfig>> {
    list_sport_configs_inner(sport_id, name_filter, limit).await
}

#[cfg(any(feature = "ssr", feature = "test-mock"))]
async fn list_sport_configs_inner(
    sport_id: Uuid,
    name_filter: Option<String>,
    limit: Option<usize>,
) -> AppResult<Vec<app_core::SportConfig>> {
    let core = expect_context::<CoreState>().as_sport_config_state();
    let configs = core
        .list_sport_configs(sport_id, name_filter.as_deref(), limit)
        .await?;
    Ok(configs)
}
