// configuration and handling of sport specific settings

use crate::utils::id_version::IdVersion;
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

