//! The port for sport-specific logic.
//!
//! This trait defines the interface that must be implemented by any "Sport Plugin".
//! It allows the core application to handle sport-specific rules for scoring,
//! timing, and ranking without needing to know the specifics of each sport.

use crate::{Match, EntrantGroupScore, SportConfig};
use serde_json::Value;
use std::{any::Any, time::Duration};
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

/// The `SportPort` trait defines the contract for a sport-specific plugin.
pub trait SportPort: Send + Sync + Any {
    /// Returns a unique identifier for the sport.
    fn id(&self) -> Uuid;

    /// Returns a user-friendly name for the sport.
    fn name(&self) -> &'static str;

    /// Provides a JSON Schema describing the structure and rules of the configuration object.
    /// The UI can use this to dynamically generate a configuration form.
    fn get_config_schema(&self) -> Value;

    /// Returns a default, valid configuration for this sport.
    /// Useful for creating a new tournament configuration from a template.
    fn get_default_config(&self) -> Value;

    /// Validates the sport-specific part of a SportConfig.
    fn validate_config_values(&self, config: &SportConfig) -> SportResult<()>;

    /// Estimates the maximum duration of a single match based on the sport-specific configuration.
    fn estimate_match_duration(&self, config: &SportConfig) -> SportResult<Duration>;

    /// Validates a final score against the rules defined in the configuration.
    fn validate_final_score(&self, config: &SportConfig, score: &Match) -> SportResult<bool>;

    /// Gathers and calculates entrant group score
    fn get_entrant_group_score(
        &self,
        group: Uuid,
        entrant_id: Uuid,
        all_matches: &[Match],
    ) -> SportResult<EntrantGroupScore>;
}
