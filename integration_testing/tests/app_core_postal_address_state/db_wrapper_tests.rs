use app_core::DbError;
use uuid::Uuid;

use integration_testing::port_fakes::*;

/// 1) load(): found → state replaced, Some returned
#[tokio::test]
async fn given_existing_id_when_load_then_state_is_replaced_and_some_is_returned() {
    let (mut core, _db_fake, _cr_fake) = make_core_with_fakes();

    // Seed via DB fake:
    let a = make_addr("Alpha", "Street 1", "10115", "Berlin", "BE", "DE");

    *core.get_mut() = a.clone();
    let id = core
        .save()
        .await
        .expect("initial save should succeed")
        .get_id()
        .unwrap();

    // Act
    let res = core.load(id).await.expect("db ok");
    assert!(res.is_some(), "should return Some(&PostalAddress)");

    // Assert state was replaced by the record from DB (version set to 0 by fake)
    let got = core.get().clone();
    assert_eq!(got.get_id(), Some(id));
    assert_eq!(got.get_name(), "Alpha");
    assert_eq!(got.get_version(), Some(0), "initial insert sets version 0");
}

/// 2) load(): not found → None, state unchanged
#[tokio::test]
async fn given_missing_id_when_load_then_none_and_state_unchanged() {
    let (mut core, _db_fake, _cr_fake) = make_core_with_fakes();

    // Prepare a known state
    let known = make_addr("Alpha", "Street 1", "10115", "Berlin", "BE", "DE");
    *core.get_mut() = known.clone();

    // Act
    let res = core.load(Uuid::new_v4()).await.expect("db ok");
    assert!(res.is_none());

    // Assert unchanged
    assert_eq!(core.get(), &known);
}

/// 3) load(): DB error propagates, state unchanged
#[tokio::test]
async fn given_db_fake_failure_when_load_then_error_propagates_and_state_unchanged() {
    let (mut core, db_fake, _cr_fake) = make_core_with_fakes();

    // Keep baseline
    let before = core.get().clone();

    // Inject failure into fake
    db_fake.fail_get_once();

    // Act
    let err = core
        .load(Uuid::new_v4())
        .await
        .expect_err("expected DB error");

    // Assert propagated and state unchanged
    match err {
        DbError::Other(e) => assert!(e.to_string().contains("injected get failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
    assert_eq!(
        core.get(),
        &before,
        "state must remain unchanged on DB error"
    );
}

/// 4) save(): persists & replaces state with DB result
#[tokio::test]
async fn given_valid_state_when_save_then_db_fake_result_replaces_state_and_is_returned() {
    let (mut core, _db_fake, _cr_fake) = make_core_with_fakes();

    // Arrange a new address in state
    core.get_mut()
        .set_name("Gamma")
        .set_street("Street 3")
        .set_postal_code("10117")
        .set_locality("Berlin")
        .set_region("BE")
        .set_country("DE");

    // Act
    let saved = core.save().await.expect("save ok").clone();

    // Assert: DB assigned version 0 on insert; core.get() equals returned ref
    assert_eq!(saved.get_version(), Some(0));
    assert_eq!(core.get(), &saved);
}

/// 5) save(): DB error propagates, state unchanged
#[tokio::test]
async fn given_db_fake_failure_when_save_then_error_propagates_and_state_unchanged() {
    let (mut core, db_fake, _cr_fake) = make_core_with_fakes();

    // Seed state
    let before = core.get().clone();

    // Act
    // Inject failure (see note in test 3)
    db_fake.fail_save_once();
    let err = core.save().await.expect_err("expected DB error");

    // Assert propagated and state unchanged
    match err {
        DbError::Other(e) => assert!(e.to_string().contains("injected save failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
    assert_eq!(
        core.get(),
        &before,
        "state must remain unchanged on DB error"
    );
}

/// 6) list_addresses(): passthrough w/ filter+limit
#[tokio::test]
async fn given_filter_and_limit_when_list_addresses_then_db_fake_results_are_forwarded() {
    let (mut core, _db_fake, _cr_fake) = make_core_with_fakes();

    // Seed via saves:
    for (nm, st, pc) in [
        ("Max", "S1", "10115"),
        ("Mara", "S2", "10116"),
        ("Zoe", "S3", "10117"),
    ] {
        *core.get_mut() = make_addr(nm, st, pc, "Berlin", "BE", "DE");
        core.save().await.expect("seed save");
    }

    // Act
    let got = core
        .list_addresses(Some("ma"), Some(2))
        .await
        .expect("db ok");

    // Assert: exactly 2 with names containing "ma" (case-insensitive)
    assert_eq!(got.len(), 2);
    let names: Vec<_> = got.iter().map(|x| x.get_name()).collect();
    assert!(names.contains(&"Mara"));
    assert!(names.contains(&"Max"));
}

/// 7) list_addresses(): only limit
#[tokio::test]
async fn given_only_limit_when_list_addresses_then_limit_is_respected() {
    let (mut core, _db_fake, _cr_fake) = make_core_with_fakes();

    for i in 0..5 {
        let nm = format!("Name{i}");
        *core.get_mut() = make_addr(&nm, "S", "P", "C", "", "DE");
        core.save().await.expect("seed save");
    }

    let got = core.list_addresses(None, Some(3)).await.expect("db ok");
    assert_eq!(got.len(), 3);
}

/// 8) list_addresses(): DB error propagates
#[tokio::test]
async fn given_db_fake_failure_when_list_addresses_then_error_propagates() {
    let (core, db_fake, _cr_fake) = make_core_with_fakes();

    // let (db_fake, _cr_fake_fake) = extract_fakes(&core);
    db_fake.fail_list_once();

    let err = core
        .list_addresses(None, None)
        .await
        .expect_err("expected DB error");

    // Assert propagated and state unchanged
    match err {
        DbError::Other(e) => assert!(e.to_string().contains("injected list failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
}
