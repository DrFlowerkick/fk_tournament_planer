// tests/client_registry_adapter_shutdown.rs
//! Shutdown semantics for the single-instance client registry adapter.
//!
//! We verify that when the *last* adapter handle is dropped, active streams
//! end promptly and no zombie tasks remain (observable by quick stream end).
//! Since after the last handle is gone we cannot call methods anymore, the
//! assertion focuses on stream termination behavior.

use integration_testing::cr_single_instance_test_support::*;
use futures_util::StreamExt;
use std::time::Duration;

/// S1: Dropping the *last* adapter handle should make active streams end quickly.
/// We publish some events, then drop the last handle, then assert the stream ends
/// within a short grace period (no zombie tasks, no hang).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_active_stream_when_last_handle_dropped_then_stream_ends_quickly()
-> anyhow::Result<()> {
    init_tracing();

    // Create a fresh adapter and subscribe.
    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();

    let mut stream = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    // Publish a couple of events to exercise the stream.
    for v in 1..=3 {
        publish_address_updated(adapter.as_ref(), id, v).await?;
        let _ = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await?
            .expect("stream ended unexpectedly before shutdown");
    }

    // Drop the last handle: stream should end soon after.
    drop(adapter);

    // After dropping the last handle, the next poll should return None
    // within a short grace period.
    let end = tokio::time::timeout(Duration::from_secs(2), async {
        while stream.next().await.is_some() {
            // It's fine to still see buffered events briefly; keep polling.
        }
    })
    .await;
    assert!(
        end.is_ok(),
        "stream did not end within grace period after last handle drop"
    );
    Ok(())
}

/// S2: If additional strong handles exist, dropping a non-final handle must NOT end streams.
/// This complements the shutdown test by ensuring correct ref-count semantics.
///
/// We keep two handles alive while streaming, drop one, and verify the stream remains alive.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_additional_handles_when_drop_non_final_then_stream_remains_alive()
-> anyhow::Result<()> {
    init_tracing();

    let adapter1 = make_adapter()?;
    let adapter2 = adapter1.clone(); // keep an extra strong handle alive

    let topic = unique_topic();
    let id = *topic.id();

    let mut stream = subscribe_with_timeout(adapter1.clone(), topic, DEFAULT_TIMEOUT).await?;

    // Drop only one handle.
    drop(adapter2);

    // Stream must still deliver new events.
    for v in 1..=5 {
        publish_address_updated(adapter1.as_ref(), id, v).await?;
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await?
            .expect("stream ended unexpectedly after dropping a non-final handle");
        assert_eq!(notice_version(&n), v);
    }

    Ok(())
}
