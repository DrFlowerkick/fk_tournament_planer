//! Basic correctness tests for the TournamentBase DB adapter.

use anyhow::Result;
use app_core::{DbError, DbpTournamentBase, TournamentState};
use integration_testing::db_postgres_test_support::{common::*, tournament_base::*};
use tracing::info;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn given_new_when_save_then_get_roundtrip_version_is_0() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Arrange
    let sport_id = Uuid::new_v4();
    let tb0 = make_new_tournament_base("A", sport_id);

    // Act: save -> DB generates id, sets version = 0
    let saved = db.save_tournament_base(&tb0).await?;
    info!(id=?saved.get_id(), v=?saved.get_version(), "saved_v0");

    // Assert basic invariants
    assert!(saved.get_id().is_some(), "id must be assigned by DB");
    assert_eq!(saved.get_version(), Some(0), "new rows start at version=0");

    // Read-back
    let fetched = db.get_tournament_base(saved.get_id().unwrap()).await?;
    assert!(fetched.is_some(), "row must exist");
    let fetched = fetched.unwrap();

    // Field-by-field sanity checks
    assert_eq!(fetched.get_version(), Some(0));
    assert_eq!(fetched.get_sport_id(), sport_id);
    assert_eq!(fetched.get_name(), "Tournament A");
    assert_eq!(fetched.get_num_entrants(), 16);
    assert_eq!(fetched.get_tournament_state(), TournamentState::Pending);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_existing_v0_when_update_then_version_increments_to_1() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Insert v0
    let sport_id = Uuid::new_v4();
    let v0 = db
        .save_tournament_base(&make_new_tournament_base("B", sport_id))
        .await?;
    assert_eq!(v0.get_version(), Some(0));

    // Prepare update using Existing(id,version=0)
    let v0_id = v0.get_id().unwrap();
    let v1_candidate = mutate_tournament_base_v2(v0.clone());

    // Act
    let v1 = db.save_tournament_base(&v1_candidate).await?;
    assert_eq!(v1.get_id(), Some(v0_id));
    assert_eq!(v1.get_version(), Some(1), "update must bump version to 1");

    // Verify persisted content
    let fetched = db.get_tournament_base(v0_id).await?.expect("row present");
    assert_eq!(fetched.get_version(), Some(1));
    assert_eq!(fetched.get_name(), "Updated Tournament V2");
    assert_eq!(
        fetched.get_tournament_state(),
        TournamentState::ActiveStage(0)
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_stale_version_when_update_then_conflict_error() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Insert v0
    let sport_id = Uuid::new_v4();
    let v0 = db
        .save_tournament_base(&make_new_tournament_base("C", sport_id))
        .await?;
    let id = v0.get_id().unwrap();
    assert_eq!(v0.get_version(), Some(0));

    // First update to v1
    let v1 = db
        .save_tournament_base(&mutate_tournament_base_v2(v0.clone()))
        .await?;
    assert_eq!(v1.get_version(), Some(1));

    // Try to update again using the *stale* v0 snapshot (Existing(id,0))
    let stale = mutate_tournament_base_v3(v0);
    let err = db
        .save_tournament_base(&stale)
        .await
        .expect_err("must conflict");

    // Pattern match the domain error
    assert!(matches!(err, DbError::OptimisticLockConflict));

    // Row remains at v1
    let fetched = db.get_tournament_base(id).await?.expect("row present");
    assert_eq!(fetched.get_version(), Some(1));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_unknown_id_when_get_then_none() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let unknown = Uuid::new_v4();
    let res = db.get_tournament_base(unknown).await?;
    assert!(res.is_none(), "unknown id should return None");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_name_filter_when_list_then_ordered_and_bounded() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let sport_id = Uuid::new_v4();
    // Filter: name contains 'ice' (case-insensitive) - matches Alice (and potentially others if they had it)
    // We update names to be distinct/simple to avoid prefix "Tournament " interference if we filtered by 'a'
    let mut t1 = make_new_tournament_base("Alice", sport_id);
    t1.set_name("Alice");
    let _ = db.save_tournament_base(&t1).await?;

    let mut t2 = make_new_tournament_base("Bob", sport_id);
    t2.set_name("Bob");
    let _ = db.save_tournament_base(&t2).await?;

    let mut t3 = make_new_tournament_base("Charlie", sport_id);
    t3.set_name("Charlie");
    let _ = db.save_tournament_base(&t3).await?;

    // Insert one for ANOTHER sport (should be ignored)
    let other_sport_id = Uuid::new_v4();
    let mut t_other = make_new_tournament_base("Alice", other_sport_id);
    t_other.set_name("Alice");
    let _ = db.save_tournament_base(&t_other).await?;

    // Filter: name contains 'a' (case-insensitive)
    // Alice (matches), Bob (no), Charlie (matches)
    let listed = db
        .list_tournament_bases(sport_id, Some("a"), Some(2))
        .await?;

    // Expect at most 2 rows, and only from sport_id
    assert!(listed.len() <= 2, "must respect limit");

    // All should belong to sport_id
    for item in &listed {
        assert_eq!(item.get_sport_id(), sport_id);
    }

    // Names are ordered ascending (NULLS LAST)
    let names: Vec<String> = listed
        .into_iter()
        .map(|t| t.get_name().to_string())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "expected name-ascending order");

    // "Alice" and "Charlie" match "a"
    // "Bob" does not.
    // So we expect Alice and Charlie if limit allows. Limit 2 covers them.
    assert!(names.iter().any(|n| n.contains("Alice")));
    assert!(names.iter().any(|n| n.contains("Charlie")));

    Ok(())
}
