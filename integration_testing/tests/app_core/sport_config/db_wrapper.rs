use app_core::DbError;
use uuid::Uuid;

use integration_testing::port_fakes::*;

/// 1) load(): found → state replaced, Some returned
#[tokio::test]
async fn given_existing_id_when_load_then_state_is_replaced_and_some_is_returned() {
    let (mut core, _db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    // Seed via DB fake:
    let sc = make_sport_config("Config A", Uuid::new_v4());

    *core.get_mut() = sc.clone();
    let id = core
        .save()
        .await
        .expect("initial save should succeed")
        .id_version
        .get_id()
        .unwrap();

    // Act
    let res = core.load(id).await.expect("db ok");
    assert!(res.is_some(), "should return Some(&SportConfig)");

    // Assert state was replaced by the record from DB (version set to 0 by fake)
    let got = core.get().clone();
    assert_eq!(got.id_version.get_id(), Some(id));
    assert_eq!(got.name, "Config A");
    assert_eq!(got.id_version.get_version(), Some(0), "initial insert sets version 0");
}

/// 2) load(): not found → None, state unchanged
#[tokio::test]
async fn given_missing_id_when_load_then_none_and_state_unchanged() {
    let (mut core, _db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    // Prepare a known state
    let known = make_sport_config("Config A", Uuid::new_v4());
    *core.get_mut() = known.clone();

    // Act
    let res = core.load(Uuid::new_v4()).await.expect("db ok");
    assert!(res.is_none());

    // Assert unchanged
    assert_eq!(core.get().name, known.name);
}

/// 3) load(): DB error propagates, state unchanged
#[tokio::test]
async fn given_db_fake_failure_when_load_then_error_propagates_and_state_unchanged() {
    let (mut core, db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    // Keep baseline
    let before = core.get().clone();

    // Inject failure into fake
    db_fake.fail_get_sc_once();

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
        core.get().name,
        before.name,
        "state must remain unchanged on DB error"
    );
}

/// 4) save(): persists & replaces state with DB result
#[tokio::test]
async fn given_valid_state_when_save_then_db_fake_result_replaces_state_and_is_returned() {
    let (mut core, _db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    // Arrange a new config in state
    core.get_mut().name = "Config B".to_string();
    core.get_mut().sport_id = Uuid::new_v4();

    // Act
    let saved = core.save().await.expect("save ok").clone();

    // Assert: DB assigned version 0 on insert; core.get() equals returned ref
    assert_eq!(saved.id_version.get_version(), Some(0));
    assert_eq!(core.get().name, saved.name);
}

/// 5) save(): DB error propagates, state unchanged
#[tokio::test]
async fn given_db_fake_failure_when_save_then_error_propagates_and_state_unchanged() {
    let (mut core, db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    // Seed state
    let before = core.get().clone();

    // Act
    // Inject failure
    db_fake.fail_save_sc_once();
    let err = core.save().await.expect_err("expected DB error");

    // Assert propagated and state unchanged
    match err {
        DbError::Other(e) => assert!(e.to_string().contains("injected save failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
    assert_eq!(
        core.get().name,
        before.name,
        "state must remain unchanged on DB error"
    );
}

/// 6) list_sport_configs(): passthrough w/ filter+limit
#[tokio::test]
async fn given_filter_and_limit_when_list_sport_configs_then_db_fake_results_are_forwarded() {
    let (mut core, _db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    let sport_id = Uuid::new_v4();
    // Seed via saves:
    for nm in ["Max Config", "Mara Config", "Zoe Config"] {
        *core.get_mut() = make_sport_config(nm, sport_id);
        core.save().await.expect("seed save");
    }

    // Act
    let got = core
        .list_sport_configs(Some("ma"), Some(2))
        .await
        .expect("db ok");

    // Assert: exactly 2 with names containing "ma" (case-insensitive)
    assert_eq!(got.len(), 2);
    let names: Vec<_> = got.iter().map(|x| x.name.as_str()).collect();
    assert!(names.contains(&"Mara Config"));
    assert!(names.contains(&"Max Config"));
}

/// 7) list_sport_configs(): only limit
#[tokio::test]
async fn given_only_limit_when_list_sport_configs_then_limit_is_respected() {
    let (mut core, _db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    let sport_id = Uuid::new_v4();
    for i in 0..5 {
        let nm = format!("Name{i}");
        *core.get_mut() = make_sport_config(&nm, sport_id);
        core.save().await.expect("seed save");
    }

    let got = core.list_sport_configs(None, Some(3)).await.expect("db ok");
    assert_eq!(got.len(), 3);
}

/// 8) list_sport_configs(): DB error propagates
#[tokio::test]
async fn given_db_fake_failure_when_list_sport_configs_then_error_propagates() {
    let (core, db_fake, _cr_fake) = make_core_sport_config_state_with_fakes();

    db_fake.fail_list_sc_once();

    let err = core
        .list_sport_configs(None, None)
        .await
        .expect_err("expected DB error");

    // Assert propagated and state unchanged
    match err {
        DbError::Other(e) => assert!(e.to_string().contains("injected list failure")),
        other => panic!("unexpected error variant: {other:?}"),
    }
}
