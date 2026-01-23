use app_core::{CoreError, CrError, CrMsg, DbError};

use integration_testing::port_fakes::*;

/// 9) save(): publishes exactly once with correct payload after successful persist
#[tokio::test]
async fn given_successful_db_save_when_save_then_publishes_exactly_once_with_correct_payload() {
    let (mut core, _db_fake, cr_fake) = make_core_stage_state_with_fakes();
    let t_id = core.get().get_tournament_id();

    // Arrange: new stage configuration in state
    core.get_mut().set_tournament_id(t_id);
    core.get_mut().set_number(0); // Valid stage number for TwoPoolStagesAndFinalStage
    core.get_mut().set_num_groups(2);

    // Act: persist (DB succeeds) → should publish once
    let saved = core
        .save()
        .await
        .expect("save should succeed (insert → version 0)");

    // Assert publish side-effect (exactly 1 notice, correct variant & payload)
    let notices = cr_fake.published();
    assert_eq!(notices.len(), 1, "exactly one publish expected after save");

    match &notices[0] {
        CrMsg::StageUpdated { id, version } => {
            let persisted_id = saved.get_id().expect("id should exist after insert");
            assert_eq!(*id, persisted_id, "published id must match saved id");
            assert_eq!(
                Some(*version),
                saved.get_version(),
                "published version must match saved version"
            );
            assert_eq!(
                saved.get_version(),
                Some(0),
                "insert should start at version 0"
            );
        }
        other => panic!("unexpected notice variant: {:?}", other),
    }
}

/// 10) save(): no publish on DB error
#[tokio::test]
async fn given_db_failure_when_save_then_no_publish_occurs() {
    let (mut core, db_fake, cr_fake) = make_core_stage_state_with_fakes();
    let t_id = core.get().get_tournament_id();

    // Arrange: ensure DB save fails once
    db_fake.fail_save_stage_once();

    // Put valid state so save attempts to persist
    core.get_mut().set_tournament_id(t_id);
    core.get_mut().set_number(1);
    core.get_mut().set_num_groups(4);

    // Act
    let err = core
        .save()
        .await
        .expect_err("save should fail due to injected DB error");

    // Assert error variant/message
    match err {
        CoreError::Db(DbError::Other(e)) => {
            assert!(e.contains("injected save failure"), "unexpected error: {e}")
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    // Assert: NO publish attempted
    let notices = cr_fake.published();
    assert!(
        notices.is_empty(),
        "no publish must occur when DB save fails"
    );
}

/// 11) save(): publish failure propagates; DB state is updated (ordering DB → publish)
#[tokio::test]
async fn given_publish_failure_after_successful_db_save_when_save_then_error_propagates_and_db_state_is_updated()
 {
    let (mut core, _db_fake, cr_fake) = make_core_stage_state_with_fakes();
    let t_id = core.get().get_tournament_id();

    // Arrange: insert a new config (DB should succeed), but inject publish failure
    core.get_mut().set_tournament_id(t_id);
    core.get_mut().set_number(2);
    core.get_mut().set_num_groups(1);

    cr_fake.fail_publish_once();

    // Act: expect error (publish fails after DB has persisted)
    let err = core
        .save()
        .await
        .expect_err("publish failure should propagate");

    // Assert error shape/message
    match err {
        CoreError::Cr(CrError::Other(e)) => assert!(
            e.to_lowercase().contains("publish") || e.to_lowercase().contains("failure"),
            "unexpected error: {e}"
        ),
        other => panic!("unexpected error variant: {other:?}"),
    }

    // DB part already succeeded → core state should now have an Existing id & version 0
    let after = core.get().clone();
    assert!(
        after.get_id().is_some(),
        "id should be set after DB success even if publish failed"
    );
    assert_eq!(
        after.get_version(),
        Some(0),
        "insert should set version 0 before publish"
    );
    // And no successful publish was recorded
    assert!(
        cr_fake.published().is_empty(),
        "publish failed; no notice should be recorded"
    );
}

/// 12) read operations never publish (load, list)
#[tokio::test]
async fn given_read_operations_when_invoked_then_never_publish_anything() {
    let (mut core, _db_fake, cr_fake) = make_core_stage_state_with_fakes();
    let t_id = core.get().get_tournament_id();

    // Seed two entries via normal saves (which *do* publish)...
    let mut s1 = core.get().clone();
    s1.set_id_version(app_core::utils::id_version::IdVersion::New);
    s1.set_tournament_id(t_id);
    s1.set_number(0);
    s1.set_num_groups(2);
    *core.get_mut() = s1;
    let saved_id = core.save().await.expect("seed 0").get_id().unwrap();

    let mut s2 = core.get().clone();
    s2.set_id_version(app_core::utils::id_version::IdVersion::New);
    s2.set_tournament_id(t_id);
    s2.set_number(1);
    s2.set_num_groups(2);
    *core.get_mut() = s2;
    core.save().await.expect("seed 1");

    // ...then clear the registry to focus purely on read calls.
    cr_fake.clear();

    // Act: load_by_id, load_by_number and list
    let _ = core.load_by_id(saved_id).await.expect("load_by_id ok");
    let _ = core.load_by_number(1).await.expect("load_by_number ok");
    let _ = core.list_stages_of_tournament().await.expect("list ok");

    // Assert: still no publish after read-only operations
    assert!(
        cr_fake.published().is_empty(),
        "read operations must not publish"
    );
}

/// 13) two consecutive saves → two publishes; versions monotonic
#[tokio::test]
async fn given_two_consecutive_saves_then_two_publishes_and_version_monotonic() {
    let (mut core, _db_fake, cr_fake) = make_core_stage_state_with_fakes();
    let t_id = core.get().get_tournament_id();

    // First insert
    core.get_mut().set_tournament_id(t_id);
    core.get_mut().set_number(0);
    core.get_mut().set_num_groups(2);

    let first = core.save().await.expect("first save").clone();
    let id = first.get_id().expect("id assigned on first save");
    assert_eq!(first.get_version(), Some(0));

    // Update same stage (simulate a change)
    core.get_mut().set_num_groups(4);

    let second = core.save().await.expect("second save (update)");
    assert_eq!(second.get_id().unwrap(), id);
    assert!(
        second.get_version() > first.get_version(),
        "update should bump version (monotonic)"
    );

    // Assert: two publish records total
    let notices = cr_fake.published();
    assert_eq!(notices.len(), 2, "two publishes expected (insert + update)");

    // (Optional) sanity on last notice payload
    match notices.last().unwrap() {
        CrMsg::StageUpdated { id: nid, version } => {
            assert_eq!(*nid, id);
            assert_eq!(Some(*version), second.get_version());
        }
        other => panic!("unexpected notice variant: {:?}", other),
    }
}
