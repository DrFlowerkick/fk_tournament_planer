// client registry port types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Display};
use thiserror::Error;
use uuid::Uuid;

/// Topics a client can subscribe to. Extend as needed for your domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CrTopic {
    Address(Uuid),
    SportConfig(Uuid),
    TournamentBase(Uuid),
    Stage(Uuid),
}

impl Display for CrTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrTopic::Address(id) => write!(f, "address: {id}"),
            CrTopic::SportConfig(id) => write!(f, "sport_config: {id}"),
            CrTopic::TournamentBase(id) => write!(f, "tournament: {id}"),
            CrTopic::Stage(id) => write!(f, "stage: {id}"),
        }
    }
}

impl CrTopic {
    pub fn id(&self) -> &Uuid {
        match self {
            CrTopic::Address(id) => id,
            CrTopic::SportConfig(id) => id,
            CrTopic::TournamentBase(id) => id,
            CrTopic::Stage(id) => id,
        }
    }
}

/// Domain notices sent to subscribed clients. Keep payloads minimal.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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
