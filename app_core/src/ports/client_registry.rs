// client registry port types

use anyhow::Result;
use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use uuid::Uuid;

/// Framework-agnostic event stream (boxed + pinned trait object).
pub type CrNoticeStream = Pin<Box<dyn Stream<Item = CrPushNotice> + Send + 'static>>;

/// Topics a client can subscribe to. Extend as needed for your domain.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CrTopic {
    Address(Uuid),
}

impl CrTopic {
    pub fn id(&self) -> &Uuid {
        match self {
            CrTopic::Address(id) => id,
        }
    }
}

impl From<&CrPushNotice> for CrTopic {
    fn from(notice: &CrPushNotice) -> Self {
        match notice {
            CrPushNotice::AddressUpdated { id, .. } => CrTopic::Address(*id),
        }
    }
}

/// Lightweight metadata to hint clients about a newer state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CrUpdateMeta {
    pub version: i64,
}

/// Domain notices sent to subscribed clients. Keep payloads minimal.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CrPushNotice {
    AddressUpdated { id: Uuid, meta: CrUpdateMeta },
}

/// client registry port trait
#[async_trait]
pub trait ClientRegistryPort: Send + Sync {
    /// Subscribe to a topic; dropping the returned stream ends the subscription (RAII).
    async fn subscribe(&self, topic: CrTopic) -> Result<CrNoticeStream>;

    /// Publish a notice to current listeners (no bus is created if none exist).
    async fn publish(&self, notice: CrPushNotice) -> Result<()>;
}
