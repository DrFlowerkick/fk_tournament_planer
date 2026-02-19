//! Sport Config Server Functions Module

#[cfg(any(feature = "ssr", feature = "test-mock"))]
use crate::error::AppError;
use crate::error::AppResult;
use app_core::SportConfig;
#[cfg(any(feature = "ssr", feature = "test-mock"))]
use app_core::{CoreState, utils::id_version::IdVersion};
use leptos::prelude::*;
//#[cfg(feature = "test-mock")]
//use leptos::{wasm_bindgen::JsCast, web_sys};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSportConfigFormData {
    pub id: Uuid,
    pub version: u32,
    pub sport_id: Uuid,
    pub name: String,
    pub config: String,
    pub intent: Option<String>,
}

#[server]
#[instrument(
    name = "sport_config.save",
    skip_all,
    fields(
        id = %form.id,
        version = form.version,
        // capture intent without logging full payloads
        intent = form.intent.as_deref().unwrap_or(""),
        // tiny hints only; avoid PII/body dumps
        name_len = form.name.len(),
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn save_sport_config(form: SaveSportConfigFormData) -> AppResult<SportConfig> {
    save_sport_config_inner(form).await
}

/*
Replace by on:submit handler for test mock, which is at the moment defined at EditSportConfig

#[cfg(feature = "test-mock")]
pub fn save_sport_config_mock_submit(
    id: Uuid,
    version: u32,
    sport_id: Uuid,
    name: String,
    config: String,
    intent: Option<String>,
) -> AppResult<SportConfig> {
    save_sport_config_inner(id, version, sport_id, name, config, intent).await
}*/

#[cfg(any(feature = "ssr", feature = "test-mock"))]
#[allow(clippy::too_many_arguments)]
pub async fn save_sport_config_inner(form: SaveSportConfigFormData) -> AppResult<SportConfig> {
    let mut core = expect_context::<CoreState>().as_sport_config_state();

    // get mut handle to wrapped SportConfig
    let mut_sc_core = core.get_mut();

    // Interpret intent
    // ToDo: we have to refactor this when switching to auto save.
    match form.intent.as_deref() {
        Some("update") => {
            // set id and version previously loaded
            if form.id.is_nil() {
                return Err(AppError::NilIdUpdate);
            }
            let id_version = IdVersion::new(form.id, Some(form.version));
            mut_sc_core.set_id_version(id_version);
            info!("saving_update");
        }
        Some("create") => {
            let id_version = IdVersion::new(form.id, None);
            mut_sc_core.set_id_version(id_version);
            info!("saving_create");
        }
        Some("copy_as_new") => {
            // set to nil id and no version to create a new copy
            let id_version = IdVersion::new(Uuid::nil(), None);
            mut_sc_core.set_id_version(id_version);
            info!("saving_copy_as_new");
        }
        _ => { /* ToDo: should we return err for unknown intent? Or how do we handle this case? */ }
    }

    // set sport config data from Form inputs
    mut_sc_core.set_name(form.name);
    mut_sc_core.set_sport_id(form.sport_id);
    mut_sc_core.set_config(serde_json::from_str(&form.config)?);

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
