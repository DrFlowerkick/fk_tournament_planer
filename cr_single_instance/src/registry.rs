// implementation of trait ClientRegistryPort

use anyhow::{Result, anyhow};
use app_core::{ClientRegistryPort, CrNoticeStream, CrPushNotice, CrTopic};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_core::Stream;
use futures_util::StreamExt;
use leptos::logging::log;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

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
        // Remove the topic bus if no receivers remain (saves memory).
        if let Some(bus) = self.buses.get(&self.topic) {
            if bus.receiver_count() == 0 {
                // release guard before remove
                drop(bus);
                self.buses.remove(&self.topic);
            }
        }
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
    fn ensure_bus(&self, topic: &CrTopic) -> broadcast::Sender<CrPushNotice> {
        self.buses
            .entry(topic.clone())
            .or_insert_with(|| {
                // Small bounded buffer; slow receivers drop oldest messages.
                let (tx, _rx) = broadcast::channel::<CrPushNotice>(128);
                tx
            })
            .clone()
    }

    /// For publishing: access an existing bus without creating a new one.
    fn get_bus(&self, topic: &CrTopic) -> Option<broadcast::Sender<CrPushNotice>> {
        self.buses.get(topic).map(|g| g.clone())
    }
}

#[async_trait]
impl ClientRegistryPort for CrSingleInstance {
    async fn subscribe(&self, topic: CrTopic) -> Result<CrNoticeStream> {
        if topic.id().is_nil() {
            return Err(anyhow!("nil uuid"));
        }
        log!("some client is subscribing to: {:?}", topic);
        let tx = self.ensure_bus(&topic);
        let rx = tx.subscribe();

        // Map BroadcastStream<Result<_, _>> -> Stream<CrPushNotice>, dropping lag errors.
        let base = BroadcastStream::new(rx).filter_map(|res| async move { res.ok() });

        // Wrap to perform RAII cleanup when the stream is dropped.
        let wrapped = CrSubscriptionStream {
            inner: Box::pin(base),
            buses: Arc::clone(&self.buses),
            topic,
        };

        Ok(Box::pin(wrapped))
    }

    async fn publish(&self, notice: CrPushNotice) -> Result<()> {
        let topic = CrTopic::from(&notice);
        log!("received topic to publish: {:?}", topic);
        if let Some(tx) = self.get_bus(&topic) {
            log!("publishing topic: {:?}", topic);
            // best-effort fan-out
            let _ = tx.send(notice);
        }
        // If there is no bus, nobody is listening; intentionally do nothing.
        Ok(())
    }
}
