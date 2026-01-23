//! Basic correctness tests for the Stage DB adapter.

use anyhow::Result;
use app_core::{DbError, DbpStage};
use integration_testing::db_postgres_test_support::{common::*, stage::*};
use tracing::info;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn given_new_when_save_then_get_roundtrip_version_is_0() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    // Arrange: Parent Tournament
    let t_id = tdb.setup_tournament().await?;
    let s0 = make_new_stage(t_id, 0);

    // Act: save
    let saved = db.save_stage(&s0).await?;
    info!(id=?saved.get_id(), v=?saved.get_version(), "saved_v0");

    // Assert basics
    assert!(saved.get_id().is_some());
    assert_eq!(saved.get_version(), Some(0));

    // Read-back by ID
    let fetched = db.get_stage_by_id(saved.get_id().unwrap()).await?;
    assert!(fetched.is_some());
    let fetched = fetched.unwrap();

    assert_eq!(fetched.get_tournament_id(), t_id);
    assert_eq!(fetched.get_number(), 0);
    assert_eq!(fetched.get_num_groups(), 2);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_existing_v0_when_update_then_version_increments_to_1() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let t_id = tdb.setup_tournament().await?;
    let v0 = db.save_stage(&make_new_stage(t_id, 1)).await?;

    // Default helper creates 2 groups. Mutate to 4.
    let v1_candidate = mutate_stage_v2(v0.clone());

    // Act
    let v1 = db.save_stage(&v1_candidate).await?;

    // Assert
    assert_eq!(v1.get_id(), v0.get_id());
    assert_eq!(v1.get_version(), Some(1));
    assert_eq!(v1.get_num_groups(), 4);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_stale_version_when_update_then_conflict_error() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let t_id = tdb.setup_tournament().await?;
    let v0 = db.save_stage(&make_new_stage(t_id, 2)).await?;

    // Update to v1
    let _v1 = db.save_stage(&mutate_stage_v2(v0.clone())).await?;

    // Try update with stale v0
    let stale = mutate_stage_v3(v0);
    let err = db.save_stage(&stale).await.expect_err("should conflict");

    assert!(matches!(err, DbError::OptimisticLockConflict));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_unknown_id_when_get_then_none() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let res = db.get_stage_by_id(Uuid::new_v4()).await?;
    assert!(res.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_existing_stage_when_get_by_number_then_found() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let t_id = tdb.setup_tournament().await?;
    let _ = db.save_stage(&make_new_stage(t_id, 5)).await?; // Stage #5

    // Act match
    let found = db.get_stage_by_number(t_id, 5).await?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().get_number(), 5);

    // Act miss matches
    let miss = db.get_stage_by_number(t_id, 99).await?;
    assert!(miss.is_none());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn given_multiple_stages_when_list_then_ordered_by_number() -> Result<()> {
    init_db_testing();
    let tdb = TestDb::new().await?;
    let db = tdb.adapter();

    let t_id = tdb.setup_tournament().await?;

    // Insert out of order
    db.save_stage(&make_new_stage(t_id, 2)).await?;
    db.save_stage(&make_new_stage(t_id, 0)).await?;
    db.save_stage(&make_new_stage(t_id, 1)).await?;

    // Act
    let stages = db.list_stages_of_tournament(t_id).await?;

    // Assert
    assert_eq!(stages.len(), 3);
    assert_eq!(stages[0].get_number(), 0);
    assert_eq!(stages[1].get_number(), 1);
    assert_eq!(stages[2].get_number(), 2);

    Ok(())
}
