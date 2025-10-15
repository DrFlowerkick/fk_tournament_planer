use app_core::{CrPushNotice, DbError};
use uuid::Uuid;

use super::fakes::*; // make_core_with_fakes, make_addr, Fake* types

/// 9) save(): publishes exactly once with correct payload after successful persist
#[tokio::test]
async fn given_successful_db_save_when_save_then_publishes_exactly_once_with_correct_payload() {
    let (mut core, _db_fake, cr_fake) = make_core_with_fakes();

    // Arrange: new address in state (insert → version 0 after save)
    *core.get_mut() = make_addr("Alpha", "Street 1", "10115", "Berlin", "BE", "DE");

    // Act: persist (DB succeeds) → should publish once
    let saved = core
        .save()
        .await
        .expect("save should succeed (insert → version 0)");

    // Assert publish side-effect (exactly 1 notice, correct variant & payload)
    let notices = cr_fake.published();
    assert_eq!(notices.len(), 1, "exactly one publish expected after save");

    match &notices[0] {
        CrPushNotice::AddressUpdated { id, meta } => {
            // saved.id should be Some(Uuid) if your PostalAddress uses IdVersion::Existing
            let persisted_id = saved.get_id().expect("id should exist after insert");
            assert_eq!(*id, persisted_id, "published id must match saved id");
            assert_eq!(
                Some(meta.version),
                saved.get_version(),
                "published version must match saved version"
            );
            assert_eq!(
                saved.get_version(),
                Some(0),
                "insert should start at version 0"
            );
        } // uncomment this, when CrPushNotice is extended
          //other => panic!("unexpected notice variant: {:?}", other),
    }
}

/// 10) save(): no publish on DB error
#[tokio::test]
async fn given_db_failure_when_save_then_no_publish_occurs() {
    let (mut core, db_fake, cr_fake) = make_core_with_fakes();

    // Arrange: ensure DB save fails once
    db_fake.fail_save_once();

    // Put something in state so that save attempts to persist
    *core.get_mut() = make_addr("Beta", "Street 2", "10247", "Berlin", "BE", "DE");

    // Act
    let err = core
        .save()
        .await
        .expect_err("save should fail due to injected DB error");

    // Assert error variant/message like in your DB tests
    match err {
        DbError::Other(e) => assert!(
            e.to_string().contains("injected save failure"),
            "unexpected error: {e}"
        ),
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
    let (mut core, _db_fake, cr_fake) = make_core_with_fakes();

    // Arrange: insert a new address (DB should succeed), but inject publish failure
    *core.get_mut() = make_addr("Gamma", "Street 3", "10117", "Berlin", "BE", "DE");
    cr_fake.fail_publish_once();

    // Act: expect error (publish fails after DB has persisted)
    let err = core
        .save()
        .await
        .expect_err("publish failure should propagate");

    // Assert error shape/message (aligned with your style)
    match err {
        DbError::Other(e) => assert!(
            e.to_string().to_lowercase().contains("publish")
                || e.to_string().to_lowercase().contains("failure"),
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
    let (mut core, _db_fake, cr_fake) = make_core_with_fakes();

    // Seed two entries via normal saves (which *do* publish)...
    *core.get_mut() = make_addr("Seed0", "S1", "10111", "Berlin", "BE", "DE");
    core.save().await.expect("seed 0");
    *core.get_mut() = make_addr("Seed1", "S2", "10112", "Berlin", "BE", "DE");
    core.save().await.expect("seed 1");

    // ...then clear the registry to focus purely on read calls.
    cr_fake.clear();

    // Act: load (existing id) and list
    let any_id = core.get().get_id().unwrap_or_else(|| Uuid::new_v4());
    let _ = core.load(any_id).await.expect("load ok");
    let _ = core.list_addresses(None, Some(10)).await.expect("list ok");

    // Assert: still no publish after read-only operations
    assert!(
        cr_fake.published().is_empty(),
        "read operations must not publish"
    );
}

/// 13) two consecutive saves → two publishes; versions monotonic
#[tokio::test]
async fn given_two_consecutive_saves_then_two_publishes_and_version_monotonic() {
    let (mut core, _db_fake, cr_fake) = make_core_with_fakes();

    // First insert
    *core.get_mut() = make_addr("Delta", "S3", "10113", "Berlin", "BE", "DE");
    let first = core.save().await.expect("first save").clone();
    let id = first.get_id().expect("id assigned on first save");
    assert_eq!(first.get_version(), Some(0));

    // Update same address (simulate a small change
    core.get_mut().set_name("Delta Renamed");

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
        CrPushNotice::AddressUpdated { id: nid, meta } => {
            assert_eq!(*nid, id);
            assert_eq!(Some(meta.version), second.get_version());
        } // uncomment this, when CrPushNotice is extended
          //other => panic!("unexpected notice variant: {:?}", other),
    }
}
