use app_core::{CoreError, DbError};
use uuid::Uuid;

use integration_testing::port_fakes::*;

/// 1) load_by_id(): found → state replaced, Some returned
#[tokio::test]
async fn given_existing_id_when_load_by_id_then_state_is_replaced_and_some_is_returned() {
    let (mut core, _db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;
    let t_id = core.get_tournament().get_id().unwrap();

    // Prepare state
    core.get_mut().set_tournament_id(t_id);
    core.get_mut().set_number(0); // Valid: < 3
    core.get_mut().set_num_groups(4); // Valid: 4 <= 32/2

    let id = core
        .save()
        .await
        .expect("initial save should succeed")
        .get_id()
        .unwrap();

    // Change state to verify reload
    core.get_mut().set_num_groups(1);

    // Act
    let res = core.load_by_id(id).await.expect("db ok");
    assert!(res.is_some(), "should return Some(&Stage)");

    // Assert state was replaced by the record from DB
    let got = core.get().clone();
    assert_eq!(got.get_id(), Some(id));
    assert_eq!(got.get_number(), 0);
    assert_eq!(got.get_num_groups(), 4);
}

/// 2) load_by_number(): found → state replaced, Some returned
#[tokio::test]
async fn given_existing_number_when_load_by_number_then_state_is_replaced_and_some_is_returned() {
    let (mut core, _db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;
    let t_id = core.get_tournament().get_id().unwrap();

    // Prepare state
    core.get_mut().set_tournament_id(t_id);
    core.get_mut().set_number(1); // Use Stage 1 (Valid: < 3)
    core.get_mut().set_num_groups(8); // Valid: 8 <= 32/2

    core.save().await.expect("initial save should succeed");

    // Change state to verify reload
    core.get_mut().set_num_groups(1);
    core.get_mut().set_number(2);

    // Act
    let res = core.load_by_number(1).await.expect("db ok");
    assert!(res.is_some(), "should return Some(&Stage)");

    // Assert state was replaced (state.tournament_id is implicit context)
    let got = core.get();
    assert_eq!(got.get_number(), 1);
    assert_eq!(got.get_num_groups(), 8);
}

/// 3) load_by_id(): not found → None, state unchanged
#[tokio::test]
async fn given_missing_id_when_load_by_id_then_none_and_state_unchanged() {
    let (mut core, _db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;

    core.get_mut().set_number(0);
    let before = core.get().clone();

    // Act
    let res = core.load_by_id(Uuid::new_v4()).await.expect("db ok");
    assert!(res.is_none());

    // Assert unchanged
    assert_eq!(core.get().get_number(), before.get_number());
}

/// 4) load_by_number(): not found → None, state unchanged
#[tokio::test]
async fn given_missing_number_when_load_by_number_then_none_and_state_unchanged() {
    let (mut core, _db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;

    core.get_mut().set_number(0);
    let before = core.get().clone();

    // Act
    let res = core.load_by_number(999).await.expect("db ok");
    assert!(res.is_none());

    assert_eq!(core.get().get_number(), before.get_number());
}

/// 5) load(): DB error propagates, state unchanged
#[tokio::test]
async fn given_db_fake_failure_when_load_then_error_propagates_and_state_unchanged() {
    let (mut core, db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;

    core.get_mut().set_number(0);
    let before = core.get().clone();

    // Inject failure into fake
    db_fake.fail_get_stage_once();

    // Act
    let err = core
        .load_by_id(Uuid::new_v4())
        .await
        .expect_err("expected DB error");

    match err {
        CoreError::Db(DbError::Other(e)) => assert!(e.contains("injected get failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
    assert_eq!(core.get().get_number(), before.get_number());
}

/// 6) save(): persists & replaces state with DB result
#[tokio::test]
async fn given_valid_state_when_save_then_db_fake_result_replaces_state_and_is_returned() {
    let (mut core, _db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;
    let t_id = core.get_tournament().get_id().unwrap();

    // Arrange
    core.get_mut().set_tournament_id(t_id);
    core.get_mut().set_number(0);
    core.get_mut().set_num_groups(2);

    // Act
    let saved = core.save().await.expect("save ok").clone();

    // Assert
    assert_eq!(saved.get_version(), Some(0));
    assert_eq!(core.get().get_id(), saved.get_id());
    assert!(core.get().get_id().is_some());
}

/// 7) save(): DB error propagates, state unchanged
#[tokio::test]
async fn given_db_fake_failure_when_save_then_error_propagates_and_state_unchanged() {
    let (mut core, db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;

    // Seed state (valid one)
    core.get_mut().set_num_groups(1);
    let before = core.get().clone();

    // Act
    db_fake.fail_save_stage_once();
    let err = core.save().await.expect_err("expected DB error");

    match err {
        CoreError::Db(DbError::Other(e)) => assert!(e.contains("injected save failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
    assert_eq!(core.get().get_id(), before.get_id());
}

/// 8) list_stages_of_tournament(): listing and sorting
#[tokio::test]
async fn given_multiple_stages_when_list_then_returned_sorted_by_number() {
    let (mut core, _db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;
    let t_id = core.get_tournament().get_id().unwrap();

    // Create 3 stages.
    // Numbers MUST be 0, 1, 2 because TwoPoolStagesAndFinalStage supports exactly 3 stages.
    let inputs = vec![
        (2, 2), // Number 2, 2 Groups
        (0, 4), // Number 0, 4 Groups
        (1, 8), // Number 1, 8 Groups
    ];

    for (num, grps) in inputs {
        let mut s = core.get().clone();
        s.set_id_version(app_core::utils::id_version::IdVersion::New);
        s.set_tournament_id(t_id);
        s.set_number(num);
        s.set_num_groups(grps);
        *core.get_mut() = s;

        core.save()
            .await
            .expect("save of stage failed - check validation rules");
    }

    // Act
    let list = core.list_stages_of_tournament().await.expect("db ok");

    // Assert
    assert_eq!(list.len(), 3);

    // check sort order (ASC by number)
    assert_eq!(list[0].get_number(), 0);
    assert_eq!(list[1].get_number(), 1);
    assert_eq!(list[2].get_number(), 2);

    // check content correctness (matches inputs above)
    assert_eq!(list[0].get_num_groups(), 4); // #0 -> 4 grps
    assert_eq!(list[2].get_num_groups(), 2); // #2 -> 2 grps
}

/// 9) list_stages_of_tournament(): DB error propagates
#[tokio::test]
async fn given_db_fake_failure_when_list_stages_then_error_propagates() {
    let (core, db_fake, _cr_fake) = make_core_stage_state_with_fakes().await;

    db_fake.fail_list_stage_once();

    let err = core
        .list_stages_of_tournament()
        .await
        .expect_err("expected DB error");

    match err {
        CoreError::Db(DbError::Other(e)) => assert!(e.contains("injected list failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
}
