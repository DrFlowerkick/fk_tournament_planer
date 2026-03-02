//! The port for sport-specific logic.
//!
//! This trait defines the interface that must be implemented by any "Sport Plugin".
//! It allows the core application to handle sport-specific rules for scoring,
//! timing, and ranking without needing to know the specifics of each sport.

use crate::{
    EntrantGroupScore, Match, SportConfig,
    utils::{
        id_version::IdVersion,
        traits::ObjectIdVersion,
        validation::{ValidationErrors, ValidationResult},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{any::Any, sync::Arc, time::Duration};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during sport-specific operations.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum SportError {
    #[error("Invalid score format: {0}")]
    InvalidScore(String),
    #[error("Unknown Sport ID: {0}")]
    UnknownSportId(Uuid),
    #[error("Invalid Sport ID: {0}, expected sport ID: {1}")]
    InvalidSportId(Uuid, Uuid),
    #[error("Invalid Json configuration: {0}")]
    InvalidJsonConfig(String),
    #[error("Configuration is invalid: {0}")]
    InvalidConfig(#[from] ValidationErrors),
    #[error("internal error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for SportError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Database Error converted to string: {:?}", err);
        Self::Other(err.to_string())
    }
}

pub type SportResult<T> = Result<T, SportError>;

impl ObjectIdVersion for Arc<dyn SportPort> {
    fn get_id_version(&self) -> IdVersion {
        self.as_ref().get_id_version()
    }
}

/// The `SportPort` trait defines the contract for a sport-specific plugin.
pub trait SportPort: ObjectIdVersion + Send + Sync + Any {
    /// Returns a user-friendly name for the sport.
    fn name(&self) -> &'static str;

    /// Returns a default, valid configuration for this sport.
    /// Useful for creating a new tournament configuration from a template.
    fn get_default_config(&self) -> Value;

    /// Validates the sport-specific part of a SportConfig.
    fn validate_config_values(
        &self,
        config: &SportConfig,
        validation_errors: ValidationErrors,
    ) -> ValidationResult<()>;

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
