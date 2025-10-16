#![cfg(feature = "ssr")]
// tests/client_registry_adapter_resilience.rs
//! Resilience tests for the single-instance client registry adapter.
//!
//! Focus here is on robustness of independent subscribers, lagging consumers,
//! and handle cloning semantics under load – without external transports.
//!
//! Notes:
//! - Comments are in English by request.
//! - These tests assume an in-memory single-process adapter with per-subscriber
//!   broadcast buffers and a drop policy on overflow.

mod common;

use common::*;
use futures_util::StreamExt;
use std::time::Duration;

/// R1: A lagging subscriber that drops (due to overflow or explicit drop)
/// must not affect other healthy subscribers on the same topic.
///
/// Strategy:
/// - Create two subscribers S_fast and S_slow on the same topic.
/// - Publish a moderate number of events while S_slow intentionally does not read.
/// - S_fast concurrently reads and must observe a full sequence (no interference).
/// - Finally drop S_slow and ensure S_fast still keeps receiving new events.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn given_two_subscribers_one_lags_when_publishing_then_other_is_healthy_and_continues()
-> anyhow::Result<()> {
    init_tracing();

    // Arrange
    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();

    let mut s_fast =
        subscribe_with_timeout(adapter.as_ref(), topic.clone(), DEFAULT_TIMEOUT).await?;
    let s_slow = subscribe_with_timeout(adapter.as_ref(), topic.clone(), DEFAULT_TIMEOUT).await?;

    // Act: publish a moderate sequence
    let k: u32 = 100;
    let mut seen = vec![false; k as usize + 1];
    let mut received = 0usize;

    for v in 1..=k {
        publish_address_updated(adapter.as_ref(), id, v).await?;
        // Let fast subscriber drain concurrently
        if v % 8 == 0 {
            // fast path: pull from s_fast to simulate an active consumer
            if let Ok(Some(n)) =
                tokio::time::timeout(Duration::from_millis(10), s_fast.next()).await
            {
                // consumed one event, keep loop tight without sleeping
                let v = notice_version(&n) as usize;
                if (1..=k as usize).contains(&v) && !seen[v] {
                    seen[v] = true;
                    received += 1;
                }
            }
        }
        // slow subscriber intentionally does nothing here
    }

    // Now read the remainder on s_fast until we have collected all K versions.
    // We do not assert order, only completeness (each version exactly once).

    // Provide a soft deadline to flush the remainder.
    let flush_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while received < k as usize && tokio::time::Instant::now() < flush_deadline {
        if let Ok(Some(n)) = tokio::time::timeout(Duration::from_millis(50), s_fast.next()).await {
            let v = notice_version(&n) as usize;
            if (1..=k as usize).contains(&v) && !seen[v] {
                seen[v] = true;
                received += 1;
            }
        } else {
            // no event immediately; loop again until deadline
        }
    }
    assert_eq!(
        received, k as usize,
        "fast subscriber did not receive all events"
    );

    // Drop the slow subscriber and ensure fast still receives further events.
    drop(s_slow);

    for v in (k + 1)..=(k + 5) {
        publish_address_updated(adapter.as_ref(), id, v).await?;
    }

    for expected in (k + 1)..=(k + 5) {
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, s_fast.next())
            .await?
            .expect("fast subscriber stream ended unexpectedly");
        assert_eq!(
            notice_version(&n),
            expected,
            "fast subscriber missed a new event after slow was dropped"
        );
    }

    Ok(())
}

/// R2: Cloned handles keep the registry alive; dropping a non-final clone must not
/// end existing streams. Only when the last handle is dropped should streams end.
/// (The "last handle dropped" behavior is verified in the shutdown suite.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_multiple_adapter_handles_when_drop_one_then_stream_keeps_running()
-> anyhow::Result<()> {
    init_tracing();

    let adapter1 = make_adapter()?;
    let adapter2 = adapter1.clone(); // second strong ref

    let topic = unique_topic();
    let id = *topic.id();

    let mut s = subscribe_with_timeout(adapter1.as_ref(), topic.clone(), DEFAULT_TIMEOUT).await?;

    // Drop one handle, but not the last
    drop(adapter2);

    // Stream must remain functional.
    for v in 1..=5 {
        publish_address_updated(adapter1.as_ref(), id, v).await?;
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, s.next())
            .await?
            .expect("stream ended unexpectedly after dropping a non-final handle");
        assert_eq!(notice_version(&n), v);
    }

    Ok(())
}

/// R3: Subscriber created before publishing must not receive any "replayed past"
/// events from another, independent adapter instance.
/// This guards against accidental global singletons across tests.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_same_topic_on_distinct_instances_when_publishing_then_no_cross_talk()
-> anyhow::Result<()> {
    init_tracing();

    // Arrange: two independent instances
    let a = make_adapter()?;
    let b = make_adapter()?;

    // Use the SAME topic id on both instances
    let topic = unique_topic();
    let id = *topic.id();

    let mut sa = subscribe_with_timeout(a.as_ref(), topic.clone(), DEFAULT_TIMEOUT).await?;
    let mut sb = subscribe_with_timeout(b.as_ref(), topic.clone(), DEFAULT_TIMEOUT).await?;

    // Act: publish on A and on B
    publish_address_updated(a.as_ref(), id, 1).await?;
    publish_address_updated(b.as_ref(), id, 2).await?;

    // Assert: A's stream sees version 1, B's stream sees version 2 (no cross talk)
    let na = tokio::time::timeout(DEFAULT_TIMEOUT, sa.next())
        .await?
        .expect("A stream ended unexpectedly");
    let nb = tokio::time::timeout(DEFAULT_TIMEOUT, sb.next())
        .await?
        .expect("B stream ended unexpectedly");

    assert_eq!(notice_version(&na), 1, "A should see its own publish only");
    assert_eq!(notice_version(&nb), 2, "B should see its own publish only");

    // Short drain: no extras
    let extra_a = drain_for(sa, std::time::Duration::from_millis(200)).await;
    let extra_b = drain_for(sb, std::time::Duration::from_millis(200)).await;
    assert_eq!(extra_a, 0);
    assert_eq!(extra_b, 0);

    Ok(())
}
