//! Basic correctness tests for the PostalAddress DB adapter.
//! Focus: insert/read roundtrip, optimistic update (version++),
//! conflict on stale version, not-found read, simple list with filter & limit.

use integration_testing::db_postgres_test_support::{common::*, postal_address::*};
use anyhow::Result;
use app_core::{DatabasePort, DbError, DbpPostalAddress};
use tracing::info;

#[tokio::test(flavor = "multi_thread")]
async fn smoke_db_connectivity_select_1() -> Result<()> {
    // Minimal connectivity probe: fail fast if DB URL/credentials are wrong.
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();
    db.ping_db().await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_new_when_save_then_get_roundtrip_version_is_0() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Arrange
    let pa0 = make_new_address("A");

    // Act: save -> DB generates id, sets version = 0
    let saved = db.save_postal_address(&pa0).await?;
    info!(id=?saved.get_id(), v=?saved.get_version(), "saved_v0");

    // Assert basic invariants
    assert!(saved.get_id().is_some(), "id must be assigned by DB");
    assert_eq!(saved.get_version(), Some(0), "new rows start at version=0");

    // Read-back
    let fetched = db.get_postal_address(saved.get_id().unwrap()).await?;
    assert!(fetched.is_some(), "row must exist");
    let fetched = fetched.unwrap();

    // Field-by-field sanity checks (you can extend if needed)
    assert_eq!(fetched.get_version(), Some(0));
    assert_eq!(fetched.get_street(), "A Street 1");
    assert_eq!(fetched.get_postal_code(), "12345");
    assert_eq!(fetched.get_locality(), "Berlin");
    assert_eq!(fetched.get_country(), "DE");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_existing_v0_when_update_then_version_increments_to_1() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Insert v0
    let v0 = db.save_postal_address(&make_new_address("B")).await?;
    assert_eq!(v0.get_version(), Some(0));

    // Prepare update using Existing(id,version=0)
    let v0_id = v0.get_id().unwrap();
    let v1_candidate = mutate_address_v2(v0.clone());
    // v1_candidate still carries Existing(id, 0) internally

    // Act
    let v1 = db.save_postal_address(&v1_candidate).await?;
    assert_eq!(v1.get_id(), Some(v0_id));
    assert_eq!(v1.get_version(), Some(1), "update must bump version to 1");

    // Verify persisted content
    let fetched = db.get_postal_address(v0_id).await?.expect("row present");
    assert_eq!(fetched.get_version(), Some(1));
    assert_eq!(fetched.get_street(), "Changed Street 99");
    assert_eq!(fetched.get_postal_code(), "54321");
    assert_eq!(fetched.get_locality(), "Potsdam");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_stale_version_when_update_then_conflict_error() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Insert v0
    let v0 = db.save_postal_address(&make_new_address("C")).await?;
    let id = v0.get_id().unwrap();
    assert_eq!(v0.get_version(), Some(0));

    // First update to v1
    let v1 = db
        .save_postal_address(&mutate_address_v2(v0.clone()))
        .await?;
    assert_eq!(v1.get_version(), Some(1));

    // Try to update again using the *stale* v0 snapshot (Existing(id,0))
    // This should hit the optimistic-lock branch and return DbError::OptimisticLockConflict.
    let stale = mutate_address_v3(v0); // still carries version=0 internally
    let err = db
        .save_postal_address(&stale)
        .await
        .expect_err("must conflict");
    // Pattern match the domain error
    assert!(matches!(err, DbError::OptimisticLockConflict));

    // Row remains at v1
    let fetched = db.get_postal_address(id).await?.expect("row present");
    assert_eq!(fetched.get_version(), Some(1));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_unknown_id_when_get_then_none() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let unknown = uuid::Uuid::new_v4();
    let res = db.get_postal_address(unknown).await?;
    assert!(res.is_none(), "unknown id should return None");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_name_filter_and_limit_when_list_then_ordered_and_bounded() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Insert three addresses with different names; the adapter orders by (name ASC NULLS LAST, created_at ASC)
    let _ = db.save_postal_address(&make_new_address("Alice")).await?;
    let _ = db.save_postal_address(&make_new_address("Bob")).await?;
    let _ = db.save_postal_address(&make_new_address("Charlie")).await?;

    // Filter: name contains 'a' (case-insensitive due to citext column)
    let listed = db.list_postal_addresses(Some("a"), Some(2)).await?;
    // Expect at most 2 rows
    assert!(listed.len() <= 2, "must respect limit");

    // Names are ordered ascending (NULLS LAST)
    let names: Vec<String> = listed
        .into_iter()
        .map(|p| p.get_name().to_string())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "expected name-ascending order");

    Ok(())
}
