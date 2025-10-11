// implementation of trait ClientRegistryPort

use anyhow::{Result, anyhow};
use app_core::{ClientRegistryPort, CrNoticeStream, CrPushNotice, CrTopic};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_core::Stream;
use futures_util::StreamExt;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::broadcast;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use tracing::{debug, info, instrument, warn};

type CrBuses = Arc<DashMap<CrTopic, broadcast::Sender<CrPushNotice>>>;

/// RAII stream wrapper that can drop the underlying receiver and
/// remove an empty topic bus when the stream goes out of scope.
struct CrSubscriptionStream {
    inner: CrNoticeStream,
    buses: CrBuses,
    topic: CrTopic,
}

impl Stream for CrSubscriptionStream {
    type Item = CrPushNotice;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl Drop for CrSubscriptionStream {
    fn drop(&mut self) {
        // Log before potential removal for better visibility.
        let mut removed = false;
        let mut receivers = 0usize;

        if let Some(bus) = self.buses.get(&self.topic) {
            receivers = bus.receiver_count();
            if receivers == 0 {
                // release guard before remove
                drop(bus);
                self.buses.remove(&self.topic);
                removed = true;
            }
        }
        debug!(%removed, receivers, topic = %self.topic, "subscription_drop");
        // Dropping the stream drops the broadcast::Receiver
    }
}

/// In-memory implementation using DashMap + tokio::broadcast.
#[derive(Clone)]
pub struct CrSingleInstance {
    // One broadcast sender per topic (created on first subscribe).
    buses: CrBuses,
}

impl CrSingleInstance {
    pub fn new() -> Self {
        Self {
            buses: Arc::new(DashMap::new()),
        }
    }

    /// Create a bus only when a client subscribes (avoid orphan buses).
    /// Create a bus only when a client subscribes (avoid orphan buses).
    fn ensure_bus(&self, topic: &CrTopic) -> broadcast::Sender<CrPushNotice> {
        let mut created = false;
        let tx = self
            .buses
            .entry(topic.clone())
            .or_insert_with(|| {
                created = true;
                // Small bounded buffer; slow receivers drop oldest messages.
                let (tx, _rx) = broadcast::channel::<CrPushNotice>(128);
                tx
            })
            .clone();
        if created {
            info!(topic = %topic, "bus_created");
        }
        tx
    }

    /// For publishing: access an existing bus without creating a new one.
    fn get_bus(&self, topic: &CrTopic) -> Option<broadcast::Sender<CrPushNotice>> {
        self.buses.get(topic).map(|g| g.clone())
    }
}

#[async_trait]
impl ClientRegistryPort for CrSingleInstance {
    #[instrument(name = "cr.subscribe", skip(self), fields(topic = %topic))]
    async fn subscribe(&self, topic: CrTopic) -> Result<CrNoticeStream> {
        if topic.id().is_nil() {
            return Err(anyhow!("nil uuid"));
        }
        let tx = self.ensure_bus(&topic);
        let rx = tx.subscribe();
        let listeners = tx.receiver_count();
        info!(listeners, "subscribed");

        // Map BroadcastStream<Result<_, _>> -> Stream<CrPushNotice>, log lagged drops.
        let base = BroadcastStream::new(rx).filter_map(|res| async move {
            match res {
                Ok(v) => Some(v),
                Err(BroadcastStreamRecvError::Lagged(n)) => {
                    // back pressure: dropped n oldest messages
                    warn!(dropped = n, "lagged_dropped");
                    None
                }
            }
        });

        // Wrap to perform RAII cleanup when the stream is dropped.
        let wrapped = CrSubscriptionStream {
            inner: Box::pin(base),
            buses: Arc::clone(&self.buses),
            topic,
        };

        Ok(Box::pin(wrapped))
    }

    #[instrument(name = "cr.publish", skip(self, notice), fields(topic = %CrTopic::from(&notice)))]
    async fn publish(&self, notice: CrPushNotice) -> Result<()> {
        let topic = CrTopic::from(&notice);
        if let Some(tx) = self.get_bus(&topic) {
            let listeners = tx.receiver_count();
            let sent = tx.send(notice).is_ok();
            debug!(listeners, %sent, "publish_attempt");
        } else {
            debug!("publish_no_listeners");
        }
        Ok(())
    }
}
