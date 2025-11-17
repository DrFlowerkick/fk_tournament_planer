// tests/client_registry_adapter_basic.rs
//! Basic P1 tests for the real Client Registry adapter.

use app_core::CrMsg;
use futures_util::StreamExt;
use integration_testing::cr_single_instance_test_support::*;
use std::time::Duration;
use uuid::Uuid;

/// P1.1: Simple Publish→Receive
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_subscribed_when_publish_then_one_notice_received() -> anyhow::Result<()> {
    init_tracing();

    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();

    // Subscribe first (hot stream: only new events)
    let mut stream = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    // Publish one notice
    publish_address_updated(adapter.as_ref(), id, 1).await?;

    // Expect exactly one notice (for this basic test)
    let n = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for first event"))?
        .expect("stream ended unexpectedly");

    match n {
        CrMsg::AddressUpdated {
            id: got_id,
            version,
        } => {
            assert_eq!(got_id, id, "received unexpected id");
            assert_eq!(version, 1, "unexpected version");
        } // uncomment this, when CrPushNotice is extended
          //_ => anyhow::bail!("unexpected notice variant"),
    }

    Ok(())
}

/// P1.2: No replay on cold subscribe
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_past_publishes_when_subscribe_then_no_replay() -> anyhow::Result<()> {
    init_tracing();

    let adapter = make_adapter()?;
    let topic = unique_topic();

    // Publish before subscribing
    let id = Uuid::new_v4();
    for v in 1..=3 {
        publish_address_updated(adapter.as_ref(), id, v).await?;
    }

    // Now subscribe; expect zero past events
    let stream = subscribe_with_timeout(adapter, topic, DEFAULT_TIMEOUT).await?;
    let received = drain_for(stream, Duration::from_millis(500)).await;
    assert_eq!(received, 0, "got unexpected replayed events");

    Ok(())
}

/// P1.3: Fan-out: Two subscribers both receive the event
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_two_subscribers_when_publish_then_both_receive_one() -> anyhow::Result<()> {
    init_tracing();

    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();

    let mut s1 = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;
    let mut s2 = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    // Publish once
    publish_address_updated(adapter.as_ref(), id, 1).await?;

    let n1 = tokio::time::timeout(DEFAULT_TIMEOUT, s1.next())
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for event on s1"))?
        .unwrap();
    let n2 = tokio::time::timeout(DEFAULT_TIMEOUT, s2.next())
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for event on s2"))?
        .unwrap();

    match (&n1, &n2) {
        (
            CrMsg::AddressUpdated {
                id: id1,
                version: v1,
            },
            CrMsg::AddressUpdated {
                id: id2,
                version: v2,
            },
        ) => {
            assert_eq!(*id1, id);
            assert_eq!(*id2, id);
            assert_eq!(*v1 as u64, 1);
            assert_eq!(*v2 as u64, 1);
        } // uncomment this, when CrPushNotice is extended
          //_ => anyhow::bail!("unexpected notice variant(s)"),
    }

    Ok(())
}

/// P1.4: Ordering is non-decreasing by `meta.version`
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_sequential_versions_when_publish_then_versions_non_decreasing() -> anyhow::Result<()>
{
    init_tracing();

    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();
    let mut stream = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    let k = 50u32;
    for v in 1..=k {
        publish_address_updated(adapter.as_ref(), id, v).await?;
    }

    // Collect k events
    let mut versions = Vec::with_capacity(k as usize);
    for i in 0..k {
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await
            .map_err(|_| anyhow::anyhow!("timed out waiting for event {i} of {k}"))?
            .unwrap();
        versions.push(notice_version(&n));
    }

    assert_non_decreasing(versions);
    Ok(())
}

/// P1.5: No spurious duplicates in steady state
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_steady_state_when_publish_sequence_then_no_spurious_duplicates() -> anyhow::Result<()>
{
    init_tracing();

    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();
    let mut stream = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    let k = 40u32;

    for v in 1..=k {
        publish_address_updated(adapter.as_ref(), id, v).await?;
    }

    // Collect exactly k events and ensure each version appears exactly once.
    let mut seen = vec![false; k as usize + 1];
    for i in 0..k {
        let n = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
            .await
            .map_err(|_| anyhow::anyhow!("timed out waiting for event {i} of {k}"))?
            .unwrap();
        let v = notice_version(&n);
        assert!((1..=k).contains(&v), "unexpected version {v}");
        assert!(!seen[v as usize], "duplicate version {v} in steady state");
        seen[v as usize] = true;
    }

    // short drain window must not show extra events
    let extra = drain_for(stream, Duration::from_millis(300)).await;
    assert_eq!(
        extra, 0,
        "received unexpected extra events after collecting k"
    );

    Ok(())
}

/// P1.6: Dropping the subscription ends delivery
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_subscription_when_drop_then_no_more_events() -> anyhow::Result<()> {
    init_tracing();

    let adapter = make_adapter()?;
    let topic = unique_topic();
    let id = *topic.id();

    // Take a subscription and receive one event
    let mut stream = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    publish_address_updated(adapter.as_ref(), id, 1).await?;
    let _first = tokio::time::timeout(DEFAULT_TIMEOUT, stream.next())
        .await?
        .unwrap();

    // Drop the stream
    drop(stream);

    // Publish more events; this subscriber cannot receive them after drop.
    for v in 2..=4 {
        publish_address_updated(adapter.as_ref(), id, v).await?;
    }

    // Fresh subscriber sees only new events
    let mut fresh = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;
    publish_address_updated(adapter.as_ref(), id, 5).await?;
    let n = tokio::time::timeout(DEFAULT_TIMEOUT, fresh.next())
        .await?
        .unwrap();
    match n {
        CrMsg::AddressUpdated { id: got, version } => {
            assert_eq!(got, id);
            assert_eq!(version, 5);
        } // uncomment this, when CrPushNotice is extended
          //_ => anyhow::bail!("unexpected notice variant"),
    }

    Ok(())
}

/// P1.7: overflow of adapter results in dropping of messages
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn given_buffer_overflow_when_publish_before_read_then_drop_policy_applies()
-> anyhow::Result<()> {
    init_tracing();

    // 1) Arrange: fresh adapter + topic/id
    let adapter = make_adapter()?;
    let topic = unique_topic(); // your helper that yields a topic with a unique id
    let id = *topic.id(); // ensure payload id matches topic id
    let stream = subscribe_with_timeout(adapter.clone(), topic, DEFAULT_TIMEOUT).await?;

    // 2) Act: publish K » buffer_capacity (e.g., 200 > 128) BEFORE reading from the stream
    // This intentionally overflows the broadcast buffer so drop policy kicks in.
    let k: u32 = 200;
    for v in 1..=k {
        publish_address_updated(adapter.as_ref(), id, v).await?;
    }

    // 3) Assert: we receive >0 but <K events due to drop policy.
    // Drain for a short window to collect what survived in the buffer.
    let received = drain_for(stream, Duration::from_millis(600)).await;

    // channel size is 128 -> expect 72 dropped message and 128 received messages
    assert!(
        received > 0 && (received as u32) < k,
        "expected some events to be dropped under overflow; got received={received}, sent={k}"
    );

    Ok(())
}
