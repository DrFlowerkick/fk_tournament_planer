// client registry port types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use thiserror::Error;
use uuid::Uuid;

/// Topics a client can subscribe to. Extend as needed for your domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CrTopic {
    NewAddress,
    Address { address_id: Uuid },
    NewSportConfig { sport_id: Uuid },
    SportConfig { sport_config_id: Uuid },
    NewTournamentBase { sport_id: Uuid },
    TournamentBase { tournament_base_id: Uuid },
    NewStage { tournament_base_id: Uuid },
    Stage { stage_id: Uuid },
}

/// Domain notices sent to subscribed clients. Keep payloads minimal.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Hash, Eq)]
pub enum CrMsg {
    AddressUpdated { id: Uuid, version: u32 },
    SportConfigUpdated { id: Uuid, version: u32 },
    TournamentBaseUpdated { id: Uuid, version: u32 },
    StageUpdated { id: Uuid, version: u32 },
}

impl CrMsg {
    pub fn id(&self) -> Uuid {
        match self {
            CrMsg::AddressUpdated { id, .. } => *id,
            CrMsg::SportConfigUpdated { id, .. } => *id,
            CrMsg::TournamentBaseUpdated { id, .. } => *id,
            CrMsg::StageUpdated { id, .. } => *id,
        }
    }

    pub fn version(&self) -> u32 {
        match self {
            CrMsg::AddressUpdated { version, .. } => *version,
            CrMsg::SportConfigUpdated { version, .. } => *version,
            CrMsg::TournamentBaseUpdated { version, .. } => *version,
            CrMsg::StageUpdated { version, .. } => *version,
        }
    }
}

/// client registry port trait
#[async_trait]
pub trait ClientRegistryPort: Send + Sync + Any {
    /// Publish a notice to current listeners (no bus is created if none exist).
    async fn publish(&self, topic: CrTopic, msg: CrMsg) -> CrResult<()>;
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum CrError {
    // Other client registry errors
    #[error("internal error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for CrError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Database Error converted to string: {:?}", err);
        Self::Other(err.to_string())
    }
}

pub type CrResult<T> = Result<T, CrError>;
