// configuration and handling of sport specific settings

use crate::{
    Core, CrMsg, CrTopic, DbResult, SportResult,
    utils::id_version::{IdVersion, VersionId},
};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// `SportConfig` represents the configuration for a specific sport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportConfig {
    /// Unique identifier for the sport configuration.
    pub id_version: IdVersion,
    /// unique combination of sport id and name of configuration
    /// sport id
    pub sport_id: Uuid,
    /// Name of the sport configuration.
    pub name: String,
    /// JSON value containing sport-specific configuration details.
    pub config: Value,
}

impl Default for SportConfig {
    fn default() -> Self {
        SportConfig {
            id_version: IdVersion::New,
            sport_id: Uuid::nil(),
            name: "".into(),
            config: Value::Null,
        }
    }
}

impl VersionId for SportConfig {
    fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
}

/// State for sport config operations
pub struct SportConfigState {
    config: SportConfig,
}

// switch state to sport config state
impl<S> Core<S> {
    pub fn as_sport_config_state(&self) -> Core<SportConfigState> {
        self.switch_state(SportConfigState {
            config: SportConfig::default(),
        })
    }
}

impl Core<SportConfigState> {
    pub fn get(&self) -> &SportConfig {
        &self.state.config
    }
    pub fn get_mut(&mut self) -> &mut SportConfig {
        &mut self.state.config
    }
    pub fn validate(&self) -> SportResult<()> {
        let sport_plugin = self
            .sport_plugins
            .get(&self.state.config.sport_id)
            .context("Invalid sport id")?;
        sport_plugin.validate_config_values(&self.state.config)?;
        Ok(())
    }

    pub async fn load(&mut self, id: Uuid) -> DbResult<Option<&SportConfig>> {
        if let Some(config) = self.database.get_sport_config(id).await? {
            self.state.config = config;
            Ok(Some(self.get()))
        } else {
            Ok(None)
        }
    }

    pub async fn save(&mut self) -> DbResult<&SportConfig> {
        self.state.config = self.database.save_sport_config(&self.state.config).await?;

        let id = self
            .state
            .config
            .id_version
            .get_id()
            .expect("expecting save_sport_config to return always an existing id and version");
        let version = self
            .state
            .config
            .id_version
            .get_version()
            .expect("expecting save_sport_config to return always an existing id and version");

        let notice = CrTopic::SportConfig(id);
        let msg = CrMsg::SportConfigUpdated { id, version };
        self.client_registry.publish(notice, msg).await?;

        Ok(self.get())
    }

    pub async fn list_sport_configs(
        &self,
        sport_id: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<SportConfig>> {
        self.database
            .list_sport_configs(sport_id, name_filter, limit)
            .await
    }
}
