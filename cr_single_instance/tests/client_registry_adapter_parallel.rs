#![cfg(all(feature = "ssr", feature = "test_support"))]
// tests/client_registry_adapter_parallel.rs
//! Parallel (P2) tests for the real Client Registry adapter.
//!
//! Focus areas:
//! - Multiple concurrent publishers for the same topic (no deadlocks; complete delivery).
//! - Parallel publishing to multiple topics with isolated subscribers (no cross-talk).
//! - Ordering is not asserted here because concurrent publishing can interleave; we assert completeness.
//!
//! Notes:
//! - Comments are in English by request.
//! - Make sure to run with `--features ssr`.

use cr_single_instance::registry::test_support::*;
use futures_util::StreamExt;

/// P2.1: Many parallel publishers on the same topic should not deadlock,
/// and the subscriber observes exactly `K` distinct versions (1..=K).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn given_many_parallel_publishers_when_send_and_read_concurrently_then_all_versions_observed()
-> anyhow::Result<()> {
    init_tracing();

    // 1) Arrange: fresh adapter + topic/id
    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();

    // Start subscriber first (hot stream). We will read concurrently.
    let stream = subscribe_with_timeout(adapter.clone(), topic.clone(), DEFAULT_TIMEOUT).await?;

    // 2) Define workload: moderate K to avoid overflow with cooperative yields.
    let n_publishers = 8usize;
    // many more versions than size of adapter (128)
    let k_versions = 10_000u32;

    // 3) Spawn consumer task that collects exactly K events while publishers run.
    // Spawn in parallel before producer, since we send many more versions than size of adapter.
    // Therefore consumer is enabled to read from stream before stream capacity is overflowed.
    // We don't assert order; only completeness (every version appears once).
    let consumer = tokio::spawn({
        let mut stream = stream; // move into task
        async move {
            let mut seen = vec![false; (k_versions as usize) + 1];
            for i in 0..k_versions {
                let n = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
                    .await
                    .map_err(|_| {
                        anyhow::anyhow!("timed out waiting for event {i} of {k_versions}")
                    })?
                    .ok_or_else(|| anyhow::anyhow!("stream ended unexpectedly at {i}"))?;

                let v = notice_version(&n) as usize;
                if v == 0 || v > k_versions as usize {
                    anyhow::bail!("unexpected version {v}");
                }
                if seen[v] {
                    anyhow::bail!("duplicate version {v}");
                }
                seen[v] = true;
            }
            // drain to ensure no extras beyond K
            let extras = drain_for(stream, std::time::Duration::from_millis(200)).await;
            anyhow::ensure!(extras == 0, "received unexpected extra events: {extras}");
            Ok::<_, anyhow::Error>(())
        }
    });

    // 4) Run publishers concurrently (cooperative to avoid overflowing the buffer).
    spawn_parallel_publishers(adapter.clone(), id, n_publishers, k_versions).await;

    // 5) Consumer must finish with exactly K observed events.
    consumer.await??;
    Ok(())
}

/// P2.2: Parallel publishing to two different topics with two subscribers:
/// each subscriber only receives its corresponding topic's stream.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn given_two_topics_and_parallel_publishers_when_send_then_isolated_streams()
-> anyhow::Result<()> {
    init_tracing();

    let adapter = make_adapter()?;

    // Two distinct topics (different ids).
    let topic_a = unique_topic();
    let id_a = *topic_a.id();
    let topic_b = unique_topic();
    let id_b = *topic_b.id();

    // Two independent subscribers.
    let mut sa = subscribe_with_timeout(adapter.clone(), topic_a, DEFAULT_TIMEOUT).await?;
    let mut sb = subscribe_with_timeout(adapter.clone(), topic_b, DEFAULT_TIMEOUT).await?;

    let n_publishers = 4usize;
    let k_versions = 100u32;

    // Run parallel publishers for both topics concurrently.
    let a = spawn_parallel_publishers(adapter.clone(), id_a, n_publishers, k_versions);
    let b = spawn_parallel_publishers(adapter.clone(), id_b, n_publishers, k_versions);
    tokio::join!(a, b);

    // Collect K on each subscriber and ensure versions 1..=K exactly once.
    let mut seen_a = vec![false; (k_versions as usize) + 1];
    let mut seen_b = vec![false; (k_versions as usize) + 1];

    for _ in 0..k_versions {
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, sa.next())
            .await?
            .expect("stream A ended unexpectedly");
        let v = notice_version(&n) as usize;
        assert!(
            (1..=k_versions as usize).contains(&v),
            "unexpected version {v} on A"
        );
        assert!(!seen_a[v], "duplicate version {v} on A");
        seen_a[v] = true;
    }

    for _ in 0..k_versions {
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, sb.next())
            .await?
            .expect("stream B ended unexpectedly");
        let v = notice_version(&n) as usize;
        assert!(
            (1..=k_versions as usize).contains(&v),
            "unexpected version {v} on B"
        );
        assert!(!seen_b[v], "duplicate version {v} on B");
        seen_b[v] = true;
    }

    // No cross-talk: versions are independent; already guaranteed by separate subscribers.
    // We add a tiny drain to ensure no extras remain in either stream.
    let extra_a = drain_for(sa, std::time::Duration::from_millis(200)).await;
    let extra_b = drain_for(sb, std::time::Duration::from_millis(200)).await;
    assert_eq!(extra_a, 0, "extra events on A");
    assert_eq!(extra_b, 0, "extra events on B");

    Ok(())
}

/// P2.3: Parallel publishers to the same topic with a fast subscriber:
/// ensures we don't trigger backpressure/drop policy in this scenario and still
/// observe all versions once.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn given_fast_subscriber_and_parallel_publishers_when_send_then_no_drop_policy_triggered()
-> anyhow::Result<()> {
    init_tracing();

    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();

    // Subscribe before publishing; do not add artificial delay on receive side.
    let mut stream = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    // Keep total throughput moderate so the subscriber can keep up.
    let n_publishers = 4usize;
    let k_versions = 120u32;

    spawn_parallel_publishers(adapter.clone(), id, n_publishers, k_versions).await;

    // Expect exactly K events, each version once.
    let mut seen = vec![false; (k_versions as usize) + 1];
    for _ in 0..k_versions {
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await?
            .expect("stream ended unexpectedly");
        let v = notice_version(&n) as usize;
        assert!(
            (1..=k_versions as usize).contains(&v),
            "unexpected version {v}"
        );
        assert!(!seen[v], "duplicate version {v}");
        seen[v] = true;
    }

    Ok(())
}
