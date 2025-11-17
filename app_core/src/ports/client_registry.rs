// client registry port types

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Display};
use uuid::Uuid;

/// Topics a client can subscribe to. Extend as needed for your domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CrTopic {
    Address(Uuid),
}

impl Display for CrTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrTopic::Address(id) => write!(f, "address: {id}"),
        }
    }
}

impl CrTopic {
    pub fn id(&self) -> &Uuid {
        match self {
            CrTopic::Address(id) => id,
        }
    }
}

/// Domain notices sent to subscribed clients. Keep payloads minimal.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum CrMsg {
    AddressUpdated { id: Uuid, version: u32 },
}

/// client registry port trait
#[async_trait]
pub trait ClientRegistryPort: Send + Sync + Any {
    /// Publish a notice to current listeners (no bus is created if none exist).
    async fn publish(&self, topic: CrTopic, msg: CrMsg) -> Result<()>;
}
