// tests/common/mod.rs
//! Test helpers for the real Client Registry adapter.
//!
//! Adjust the `use` paths below to your actual crate/module structure.
//! These helpers assume the following API exists in your crate:
//!   - Trait `ClientRegistryPort`
//!   - Types: `CrTopic`, `CrPushNotice`, `CrUpdateMeta`, `CrNoticeStream`
//!   - A *test-only* factory or support module to construct topics and an adapter handle.
//!
//! Recommended to provide (in your crate under cfg(test)):
//!   - `cr_single_instance::registry::test_support::make_adapter()` -> anyhow::Result<Arc<dyn ClientRegistryPort>>
//!   - `cr_single_instance::registry::test_support::make_topic<S: AsRef<str>>(s: S) -> CrTopic`
//!   - `cr_single_instance::registry::test_support::build_address_updated(id: uuid::Uuid, version: u64) -> CrPushNotice`
//!
//! See the TODOs below if names differ in your codebase.

use super::CrSingleInstance;
use anyhow::Result;
use app_core::{ClientRegistryPort, CrNoticeStream, CrPushNotice, CrTopic, CrUpdateMeta};
use futures_util::{StreamExt, future::join_all};
use std::{
    sync::{Arc, Once},
    time::Duration,
};
use tokio::time::timeout;
use uuid::Uuid;

/// init tracing once for all test cases
/// insert common::init_tracing(); at the start of each test case
static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            // Respect RUST_LOG if set, otherwise reasonable default:
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,client_registry=debug".into()),
            )
            // Emit per-test-friendly output:
            .with_test_writer()
            .try_init();
    });
}

/// Default per-assertion timeout used by helpers.
/// Keep this generous for CI stability; individual tests can override.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Build a minimal `AddressUpdated` notice for tests.
/// Extend this if your real enum carries additional fields.
pub fn build_address_updated(id: Uuid, version: u32) -> CrPushNotice {
    CrPushNotice::AddressUpdated {
        id,
        meta: CrUpdateMeta { version },
    }
}

/// Extract the version from a `CrPushNotice`.
pub fn notice_version(n: &CrPushNotice) -> u32 {
    match n {
        CrPushNotice::AddressUpdated { meta, .. } => meta.version,
        // add more variants if needed
    }
}

/// Construct the real adapter used in tests.
pub fn make_adapter() -> Result<Arc<dyn ClientRegistryPort>> {
    Ok(Arc::new(CrSingleInstance::new()))
}

/// Produce a unique topic name by suffixing with a UUID v4.
pub fn unique_topic() -> CrTopic {
    CrTopic::Address(Uuid::new_v4())
}

/// Drain a stream for `window` duration and return how many items arrived.
pub async fn drain_for(mut stream: CrNoticeStream, window: Duration) -> usize {
    let deadline = tokio::time::Instant::now() + window;
    let mut count = 0usize;
    loop {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline - now;
        match timeout(remaining, stream.next()).await {
            Ok(Some(_)) => count += 1,
            Ok(None) => break, // stream ended
            Err(_) => break,   // no more within window
        }
    }
    count
}

/// Assert that versions are non-decreasing.
pub fn assert_non_decreasing<I, T>(versions: I)
where
    I: IntoIterator<Item = T>,
    T: Into<u64>,
{
    let mut prev: Option<u64> = None;
    for v in versions.into_iter().map(Into::into) {
        if let Some(p) = prev {
            assert!(v >= p, "version order violation: got {v}, previous {p}");
        }
        prev = Some(v);
    }
}

/// Convenience to publish a single `AddressUpdated` notice.
pub async fn publish_address_updated(
    reg: &dyn ClientRegistryPort,
    id: Uuid,
    version: u32,
) -> anyhow::Result<()> {
    let notice = build_address_updated(id, version);
    reg.publish(notice).await
}

/// Subscribe helper with timeout protection on the subscription call.
pub async fn subscribe_with_timeout(
    reg: &dyn ClientRegistryPort,
    topic: CrTopic,
    deadline: Duration,
) -> anyhow::Result<CrNoticeStream> {
    timeout(deadline, reg.subscribe(topic))
        .await
        .map_err(|_| anyhow::anyhow!("subscribe() timed out after {deadline:?}"))?
}

/// Spawn N publisher tasks that each publish a disjoint slice of version numbers.
/// The union of all slices covers `1..=total_versions`.
pub async fn spawn_parallel_publishers(
    adapter: Arc<dyn ClientRegistryPort>,
    id: Uuid,
    n_publishers: usize,
    total_versions: u32,
) {
    // Partition versions among publishers by round-robin to maximize interleaving.
    let mut tasks = Vec::with_capacity(n_publishers);
    for p in 0..n_publishers {
        let adapter_cloned = adapter.clone();
        let start = p as u32 + 1;
        let step = n_publishers as u32;
        let id_cloned = id;
        tasks.push(tokio::spawn(async move {
            let mut v = start;
            while v <= total_versions {
                // Publish a single event for version v.
                if let Err(e) = publish_address_updated(adapter_cloned.as_ref(), id_cloned, v).await
                {
                    panic!("publisher {p} failed to publish version {v}: {e:?}");
                }
                // short sleep every 16 publish to simulate system latency for writing data
                if (v / step).is_multiple_of(16) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(3)).await;
                }
                v += step;
            }
        }));
    }
    let _ = join_all(tasks).await;
}
