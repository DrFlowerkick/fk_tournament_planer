// configuration and handling of sport specific settings

use crate::{
    Core, CoreError, CoreResult, CrMsg, CrTopic, SportError,
    utils::{
        id_version::{IdVersion, VersionId},
        normalize::normalize_ws,
        validation::*,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// `SportConfig` represents the configuration for a specific sport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SportConfig {
    /// Unique identifier for the sport configuration.
    id_version: IdVersion,
    /// unique combination of sport id and name of configuration
    /// sport id
    sport_id: Uuid,
    /// Name of the sport configuration.
    name: String,
    /// JSON value containing sport-specific configuration details.
    config: Value,
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

impl SportConfig {
    /// Create a new `SportConfig` with the given `IdVersion`.
    pub fn new(id_version: IdVersion) -> Self {
        SportConfig {
            id_version,
            ..Default::default()
        }
    }

    /// Get the unique identifier of the sport configuration.
    pub fn get_id(&self) -> Option<Uuid> {
        self.id_version.get_id()
    }

    /// Get the version number of the sport configuration.
    pub fn get_version(&self) -> Option<u32> {
        self.id_version.get_version()
    }

    pub fn get_id_version(&self) -> IdVersion {
        self.id_version
    }

    /// Get the sport ID associated with this configuration.
    pub fn get_sport_id(&self) -> Uuid {
        self.sport_id
    }

    /// Get the name of the sport configuration.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the sport-specific configuration details as a JSON value.
    pub fn get_config(&self) -> &Value {
        &self.config
    }

    /// Set the `IdVersion` of the sport configuration.
    pub fn set_id_version(&mut self, id_version: IdVersion) -> &mut Self {
        self.id_version = id_version;
        self
    }

    /// Set the sport ID associated with this configuration.
    pub fn set_sport_id(&mut self, sport_id: Uuid) -> &mut Self {
        self.sport_id = sport_id;
        self
    }

    /// Set the name of the sport configuration with normalization
    /// - trims leading/trailing whitespace
    /// - collapses internal runs of whitespace to a single space
    ///
    /// # Examples
    ///
    /// ```
    /// use app_core::SportConfig;
    ///
    /// // Start from default.
    /// let mut config = SportConfig::default();
    ///
    /// // Regularize spacing (trim + collapse):
    /// config.set_name("  Fun   Sport  Config  ".to_string());
    /// assert_eq!(config.get_name(), "Fun Sport Config");
    /// ```
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = normalize_ws(name.into());
        self
    }

    /// Set the sport-specific configuration details.
    pub fn set_config(&mut self, config: Value) -> &mut Self {
        self.config = config;
        self
    }

    /// Validate the sport configuration.
    /// At this level we can only validate the name.
    /// Sport-specific validation must be done in the SportPort implementation.
    pub fn validate(&self) -> ValidationResult<()> {
        let mut errs = ValidationErrors::new();
        if self.name.is_empty() {
            errs.add(
                FieldError::builder()
                    .set_field(String::from("name"))
                    .add_required()
                    .build(),
            );
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        Ok(())
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
    fn validate(&self, config: &SportConfig) -> CoreResult<()> {
        let Some(sport_plugin) = self.sport_plugins.get(&config.sport_id) else {
            return Err(CoreError::from(SportError::UnknownSportId(config.sport_id)));
        };
        let err = match config.validate() {
            Ok(_) => ValidationErrors::new(),
            Err(e) => e,
        };
        sport_plugin.validate_config_values(config, err)?;
        Ok(())
    }
    pub async fn load(&mut self, id: Uuid) -> CoreResult<Option<&SportConfig>> {
        if let Some(config) = self.database.get_sport_config(id).await? {
            self.state.config = config;
            self.validate(&self.state.config)?;

            Ok(Some(self.get()))
        } else {
            Ok(None)
        }
    }

    pub async fn save(&mut self) -> CoreResult<&SportConfig> {
        // validate before save
        self.validate(&self.state.config)?;
        // persist config
        self.state.config = self.database.save_sport_config(&self.state.config).await?;
        // publish change of sport config to client registry
        let id = self
            .state
            .config
            .get_id()
            .expect("expecting save_sport_config to return always an existing id and version");
        let version = self
            .state
            .config
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
    ) -> CoreResult<Vec<SportConfig>> {
        let list = self
            .database
            .list_sport_configs(sport_id, name_filter, limit)
            .await?;
        for config in &list {
            self.validate(config)?;
        }
        Ok(list)
    }
}
