//! Concurrency tests for the Stage DB adapter.

use anyhow::Result;
use app_core::{DbError, DbpStage};
use integration_testing::db_postgres_test_support::{common::*, stage::*};
use tokio::sync::Barrier;

#[tokio::test(flavor = "multi_thread")]
async fn given_two_parallel_updates_from_v0_then_only_one_succeeds_and_version_is_1() -> Result<()>
{
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let t_id = tdb.setup_tournament().await?;
    let v0 = db.save_stage(&make_new_stage(t_id, 0)).await?;
    let id = v0.get_id();

    // Prepare competitors
    let candidate_a = mutate_stage_v2(v0.clone());
    let candidate_b = mutate_stage_v3(v0.clone());

    let barrier = std::sync::Arc::new(Barrier::new(2));
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let db1 = db.clone();
    let ca = candidate_a.clone();
    let h1 = tokio::spawn(async move {
        b1.wait().await;
        db1.save_stage(&ca).await
    });

    let db2 = db.clone();
    let cb = candidate_b.clone();
    let h2 = tokio::spawn(async move {
        b2.wait().await;
        db2.save_stage(&cb).await
    });

    let r1 = h1.await.expect("task1 panicked");
    let r2 = h2.await.expect("task2 panicked");

    let ok_count = (r1.is_ok() as u8) + (r2.is_ok() as u8);
    assert_eq!(ok_count, 1, "exactly one update succeeds");

    // Loser gets OptimisticLockConflict
    let loser_err = r1.err().or(r2.err()).expect("one error");
    assert!(matches!(loser_err, DbError::OptimisticLockConflict));

    // Validated DB State
    let fetched = db.get_stage_by_id(id).await?.unwrap();
    assert_eq!(fetched.get_version(), Some(1));

    let is_a = same_semantics(&fetched, &candidate_a);
    let is_b = same_semantics(&fetched, &candidate_b);
    assert!(is_a ^ is_b);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_two_parallel_inserts_same_number_then_unique_violation() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let t_id = tdb.setup_tournament().await?;

    // Two stages for same tournament, same number (0)
    let new_a = make_new_stage(t_id, 0);
    let new_b = new_a.clone(); // same content

    let barrier = std::sync::Arc::new(Barrier::new(2));
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let db1 = db.clone();
    let h1 = tokio::spawn(async move {
        b1.wait().await;
        db1.save_stage(&new_a).await
    });

    let db2 = db.clone();
    let h2 = tokio::spawn(async move {
        b2.wait().await;
        db2.save_stage(&new_b).await
    });

    let r1 = h1.await.expect("t1 panicked");
    let r2 = h2.await.expect("t2 panicked");

    let ok_count = (r1.is_ok() as u8) + (r2.is_ok() as u8);
    assert_eq!(
        ok_count, 1,
        "exactly one insert succeeds for same (t_id, number)"
    );

    let loser_err = r1.err().or(r2.err()).expect("one error");
    assert!(matches!(loser_err, DbError::UniqueViolation(_)));

    Ok(())
}
