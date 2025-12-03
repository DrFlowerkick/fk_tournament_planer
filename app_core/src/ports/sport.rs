//! The port for sport-specific logic.
//!
//! This trait defines the interface that must be implemented by any "Sport Plugin".
//! It allows the core application to handle sport-specific rules for scoring,
//! timing, and ranking without needing to know the specifics of each sport.

use crate::{
    EntrantGroupScore, Match, SportConfig,
    utils::id_version::{IdVersion, VersionId},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{any::Any, sync::Arc, time::Duration};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during sport-specific operations.
#[derive(Debug, Error)]
pub enum SportError {
    #[error("Invalid score format: {0}")]
    InvalidScore(String),
    #[error("Configuration is missing or invalid: {0}")]
    InvalidConfig(String),
    #[error("An unexpected error occurred: {0}")]
    Other(#[from] anyhow::Error),
}

pub type SportResult<T> = Result<T, SportError>;

/// Information about a sport plugin.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SportPluginInfo {
    id_version: IdVersion,
    name: String,
}

impl VersionId for SportPluginInfo {
    fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
}

impl<T: SportPort + ?Sized> From<&T> for SportPluginInfo {
    fn from(port: &T) -> Self {
        SportPluginInfo {
            id_version: port.get_id_version(),
            name: port.name().to_string(),
        }
    }
}

impl From<&Arc<dyn SportPort>> for SportPluginInfo {
    fn from(port: &Arc<dyn SportPort>) -> Self {
        // Delegiert an die generische Implementierung via Dereferenzierung (as_ref)
        Self::from(port.as_ref())
    }
}

impl SportPluginInfo {
    /// Returns the name of the sport plugin.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

/// The `SportPort` trait defines the contract for a sport-specific plugin.
pub trait SportPort: VersionId + Send + Sync + Any {
    /// Returns a user-friendly name for the sport.
    fn name(&self) -> &'static str;

    /// Returns a default, valid configuration for this sport.
    /// Useful for creating a new tournament configuration from a template.
    fn get_default_config(&self) -> Value;

    /// Validates the sport-specific part of a SportConfig.
    fn validate_config_values(&self, config: &SportConfig) -> SportResult<()>;

    /// Estimates the maximum duration of a single match based on the sport-specific configuration.
    fn estimate_match_duration(&self, config: &SportConfig) -> SportResult<Duration>;

    /// Validates a final score against the rules defined in the configuration.
    fn validate_final_score(&self, config: &SportConfig, score: &Match) -> SportResult<()>;

    /// Gathers and calculates entrant group score
    fn get_entrant_group_score(
        &self,
        config: &SportConfig,
        group_id: Uuid,
        entrant_id: Uuid,
        all_matches: &[Match],
    ) -> SportResult<EntrantGroupScore>;
}
