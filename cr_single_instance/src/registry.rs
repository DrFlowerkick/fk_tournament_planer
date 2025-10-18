// implementation of trait ClientRegistryPort

use anyhow::{Result, anyhow};
use app_core::{ClientRegistryPort, CrNoticeStream, CrPushNotice, CrTopic};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_core::Stream;
use futures_util::StreamExt;
use std::{
    pin::Pin,
    sync::{Arc, Weak},
    task::{Context, Poll},
};
use tokio::sync::broadcast;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use tracing::{debug, info, instrument, warn};

type CrBuses = Arc<DashMap<CrTopic, broadcast::Sender<CrPushNotice>>>;
type WeakBuses = Weak<DashMap<CrTopic, broadcast::Sender<CrPushNotice>>>;

/// RAII stream wrapper that can drop the underlying receiver and
/// remove an empty topic bus when the stream goes out of scope.
struct CrSubscriptionStream {
    inner: Option<CrNoticeStream>,
    buses: WeakBuses,
    topic: CrTopic,
}

impl Stream for CrSubscriptionStream {
    type Item = CrPushNotice;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut() {
            Some(inner) => inner.as_mut().poll_next(cx),
            None => Poll::Ready(None),
        }
    }
}

impl Drop for CrSubscriptionStream {
    fn drop(&mut self) {
        // Log before potential removal for better visibility.
        let mut removed = false;
        let mut receivers = 0usize;
        // drop inner to drop receiver in inner stream
        self.inner.take();

        if let Some(strong_buses) = self.buses.upgrade()
            && let Some(bus) = strong_buses.get(&self.topic)
        {
            receivers = bus.receiver_count();
            if receivers == 0 {
                // release guard before remove
                drop(bus);
                strong_buses.remove(&self.topic);
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

impl Default for CrSingleInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl CrSingleInstance {
    pub fn new() -> Self {
        Self {
            buses: Arc::new(DashMap::new()),
        }
    }

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
        let buses = Arc::clone(&self.buses);
        let wrapped = CrSubscriptionStream {
            inner: Some(Box::pin(base)),
            buses: Arc::downgrade(&buses),
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

/// support for unit and integration tests
#[cfg(any(test, feature = "test_support"))]
pub mod test_support;

#[cfg(test)]
mod tests_drop_semantics {
    use super::{*, test_support::build_address_updated};
    use futures_util::StreamExt;
    use tokio::time::{Duration, timeout};
    use uuid::Uuid;

    /// U1: Dropping the subscription should release the last receiver so that
    /// the bus can be removed (no receivers remain).
    ///
    /// Expected: after dropping the only subscription, the DashMap has no entry for the topic.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_subscription_removes_empty_bus() {
        // Arrange: fresh adapter and a unique topic
        let adapter = CrSingleInstance::new();
        let id = Uuid::new_v4();
        let topic = CrTopic::Address(id);

        // First subscribe to create the bus.
        let mut stream = adapter
            .subscribe(topic.clone())
            .await
            .expect("subscribe failed");

        // Publish one event to ensure the bus actually exists and works.
        adapter
            .publish(build_address_updated(id, 1))
            .await
            .expect("publish failed");
        let _first = timeout(Duration::from_secs(2), stream.next())
            .await
            .expect("recv timed out")
            .expect("stream ended unexpectedly");

        // At this point there is exactly one receiver (our `stream`).
        // Act: drop the stream (should drop the underlying broadcast::Receiver FIRST).
        drop(stream);

        // Assert: the bus for `topic` is gone (or at least no receivers remain).
        // Preferred invariant: entry removed.
        let contains = adapter.buses.contains_key(&topic);
        assert!(
            !contains,
            "expected bus for topic to be removed after last receiver dropped"
        );
    }

    /// U2: Dropping the last adapter handle should make active streams end quickly.
    ///
    /// This asserts "contract A": after the last strong handle to the registry is dropped,
    /// all underlying broadcast senders are dropped via Arc semantics, therefore
    /// receivers observe stream termination (`None`) promptly.
    ///
    /// Precondition for this to pass without explicit Drop on the adapter:
    ///  - streams hold only Weak to the buses (so they can't keep the map alive),
    ///  - there are no hidden Sender clones lingering elsewhere.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dropping_last_adapter_closes_stream() {
        // Arrange: create adapter and subscribe
        let adapter = CrSingleInstance::new();
        let id = Uuid::new_v4();
        let topic = CrTopic::Address(id);

        let mut stream = adapter
            .subscribe(topic.clone())
            .await
            .expect("subscribe failed");

        // Prove the stream is live
        adapter
            .publish(build_address_updated(id, 1))
            .await
            .expect("publish failed");
        let _ = timeout(Duration::from_secs(2), stream.next())
            .await
            .expect("recv timed out before shutdown")
            .expect("stream ended too early");

        // Act: drop the LAST strong handle to the registry
        drop(adapter);

        // Assert: the next poll eventually returns None within a short grace period.
        let ended = timeout(Duration::from_secs(2), async {
            loop {
                match stream.next().await {
                    Some(_) => {
                        // It's acceptable if a buffered event sneaks in; keep polling.
                        continue;
                    }
                    None => break,
                }
            }
        })
        .await;

        assert!(
            ended.is_ok(),
            "stream did not end within the grace period after the last handle was dropped"
        );
    }
}
