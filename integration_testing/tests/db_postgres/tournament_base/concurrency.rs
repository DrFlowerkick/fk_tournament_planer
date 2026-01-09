//! Concurrency tests for the TournamentBase DB adapter.

use anyhow::Result;
use app_core::{DbError, DbpTournamentBase};
use integration_testing::db_postgres_test_support::{common::*, tournament_base::*};
use tokio::sync::Barrier;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn given_two_parallel_updates_from_v0_then_only_one_succeeds_and_version_is_1() -> Result<()>
{
    // Arrange: fresh DB + seed v0
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let sport_id = Uuid::new_v4();
    let v0 = db
        .save_tournament_base(&make_new_tournament_base("concurrency-u1", sport_id))
        .await?;
    assert_eq!(v0.get_version(), Some(0));
    let id = v0.get_id().expect("id must be present");

    // Prepare two competing updates from the SAME v0 snapshot.
    let candidate_a = mutate_tournament_base_v2(v0.clone());
    let candidate_b = mutate_tournament_base_v3(v0.clone());

    // Coordinate near-simultaneous start
    let barrier = std::sync::Arc::new(Barrier::new(2));
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let db1 = db.clone();
    let candidate_a2 = candidate_a.clone();
    let h1 = tokio::spawn(async move {
        b1.wait().await;
        db1.save_tournament_base(&candidate_a2).await
    });

    let db2 = db.clone();
    let candidate_b2 = candidate_b.clone();
    let h2 = tokio::spawn(async move {
        b2.wait().await;
        db2.save_tournament_base(&candidate_b2).await
    });

    // Act
    let r1 = h1.await.expect("task1 panicked");
    let r2 = h2.await.expect("task2 panicked");

    // Assert: exactly one update succeeds
    let ok_count = (r1.is_ok() as u8) + (r2.is_ok() as u8);
    assert_eq!(ok_count, 1, "exactly one concurrent update must succeed");

    // Winner must be version 1 on same ID
    if let Ok(winner) = r1.as_ref().or(r2.as_ref()) {
        assert_eq!(winner.get_id(), Some(id));
        assert_eq!(winner.get_version(), Some(1));
    }

    // Loser must be optimistic lock domain error
    let loser_err = r1.err().or(r2.err()).expect("one loser error expected");
    assert!(matches!(loser_err, DbError::OptimisticLockConflict));

    // Final state: version == 1 and content equals exactly one of the candidates
    let fetched = db.get_tournament_base(id).await?.expect("row must exist");
    assert_eq!(fetched.get_version(), Some(1));
    let equals_a = same_semantics(&fetched, &candidate_a);
    let equals_b = same_semantics(&fetched, &candidate_b);
    assert!(
        equals_a ^ equals_b,
        "final content must match exactly one winner (A xor B)"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_two_parallel_inserts_same_name_then_only_one_succeeds_unique_violation_for_loser()
-> Result<()> {
    // Arrange: fresh DB
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Unique Constraint is on (sport_id, name).
    let sport_id = Uuid::new_v4();
    let new_a = make_new_tournament_base("concurrency-insert", sport_id);
    let new_b = new_a.clone();
    let third = new_a.clone();

    // Coordinate near-simultaneous inserts.
    let barrier = std::sync::Arc::new(Barrier::new(2));
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let db1 = db.clone();
    let h1 = tokio::spawn(async move {
        b1.wait().await;
        db1.save_tournament_base(&new_a).await
    });

    let db2 = db.clone();
    let h2 = tokio::spawn(async move {
        b2.wait().await;
        db2.save_tournament_base(&new_b).await
    });

    // Act
    let r1 = h1.await.expect("task1 panicked");
    let r2 = h2.await.expect("task2 panicked");

    // Assert: exactly one insert must succeed
    let ok_count = (r1.is_ok() as u8) + (r2.is_ok() as u8);
    assert_eq!(
        ok_count, 1,
        "exactly one parallel insert must succeed for the same (sport_id, name)"
    );

    // Loser should map to UniqueViolation
    let loser_err = r1.err().or(r2.err()).expect("one loser error expected");
    assert!(matches!(loser_err, DbError::UniqueViolation(_)));

    // a *third* insert with the same pair must also fail.
    let third_res = db
        .save_tournament_base(&third)
        .await
        .expect_err("third must result in error.");
    assert!(matches!(third_res, DbError::UniqueViolation(_)));

    Ok(())
}
