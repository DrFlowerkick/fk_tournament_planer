// client registry port types

use std::{pin::Pin};
use futures_core::Stream;
use anyhow::Result;
use uuid::Uuid;
use async_trait::async_trait;

/// Framework-agnostic event stream (boxed + pinned trait object).
pub type CrNoticeStream = Pin<Box<dyn Stream<Item = CrPushNotice> + Send + 'static>>;

/// Topics a client can subscribe to. Extend as needed for your domain.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CrTopic {
    Address(Uuid),
}

impl From<&CrPushNotice> for CrTopic {
    fn from(notice: &CrPushNotice) -> Self {
        match notice {
            CrPushNotice::AddressUpdated { id, .. } => CrTopic::Address(*id),
        }
    }
}

/// Lightweight metadata to hint clients about a newer state.
#[derive(Clone, Debug)]
pub struct CrUpdateMeta {
    pub version: i64,
}

/// Domain notices sent to subscribed clients. Keep payloads minimal.
#[derive(Clone, Debug)]
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
