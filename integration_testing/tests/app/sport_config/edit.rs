use crate::common::{
    get_element_by_test_id, get_test_root, init_test_state, lock_test, set_input_value, set_url,
};
use app::{provide_global_state, sport_config::SportConfigForm};
use app_core::DbpSportConfig;
use generic_sport_plugin::config::GenericSportConfig;
use gloo_timers::future::sleep;
use leptos::{mount::mount_to, prelude::*, wasm_bindgen::JsCast, web_sys::HtmlInputElement};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_new_sport_config() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set initial URL for creating a new sport config
    set_url(&format!("/sport/new_sc?sport_id={}", ts.generic_sport_id));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <SportConfigForm />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    // create a new sport config by filling the form
    set_input_value("input-name", "New Sport Config");
    // other fields can be left as default for this test

    sleep(Duration::from_millis(10)).await;
    let save_button = get_element_by_test_id("btn-save");
    save_button.click();

    sleep(Duration::from_millis(10)).await;

    let new_configs = ts
        .db
        .list_sport_configs(ts.generic_sport_id, Some("New"), None)
        .await
        .unwrap();
    assert_eq!(new_configs.len(), 1);
    assert_eq!(new_configs[0].get_name(), "New Sport Config");
}

#[wasm_bindgen_test]
async fn test_edit_sport_config() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set URL with sport_id
    set_url(&format!(
        "/sport/edit_sc?sport_id={}&sport_config_id={}",
        ts.generic_sport_id, ts.generic_sport_config_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <SportConfigForm />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    // verify that the form is populated with existing data
    let name_input = get_element_by_test_id("input-name")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    assert_eq!(name_input.value(), "Test Config 1");

    // modify some data and save
    set_input_value("input-victory_points_win", "5");

    sleep(Duration::from_millis(10)).await;
    let save_button = get_element_by_test_id("btn-save");
    save_button.click();

    sleep(Duration::from_millis(10)).await;
    let updated_config = ts
        .db
        .get_sport_config(ts.generic_sport_config_id)
        .await
        .unwrap()
        .unwrap();
    let updated_config_data: GenericSportConfig =
        serde_json::from_value(updated_config.get_config().clone()).unwrap();
    assert_eq!(updated_config_data.victory_points_win, 5.0);
    assert_eq!(updated_config.get_version().unwrap(), 1);

    // now save existing sport config as new
    set_input_value("input-name", "Cloned Config");

    sleep(Duration::from_millis(10)).await;

    let save_as_new_button = get_element_by_test_id("btn-save-as-new");
    save_as_new_button.click();
    sleep(Duration::from_millis(10)).await;
    let cloned_configs = ts
        .db
        .list_sport_configs(ts.generic_sport_id, Some("Cloned"), None)
        .await
        .unwrap();
    assert_eq!(cloned_configs.len(), 1);
    assert_eq!(cloned_configs[0].get_name(), "Cloned Config");
    assert_eq!(updated_config.get_version().unwrap(), 1);
    assert_eq!(cloned_configs[0].get_version().unwrap(), 0);
}
