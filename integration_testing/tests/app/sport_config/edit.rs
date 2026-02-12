use crate::common::{
    get_element_by_test_id, get_test_root, init_test_state, lock_test, set_input_value, set_url,
};
use app::{home::LoadSportConfiguration, provide_global_context};
use app_core::{DbpSportConfig, SportConfig};
use app_utils::{params::SportConfigIdQuery, state::object_table_list::ObjectListContext};
use generic_sport_plugin::config::GenericSportConfig;
use gloo_timers::future::sleep;
use leptos::{mount::mount_to, prelude::*, wasm_bindgen::JsCast, web_sys::HtmlInputElement};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use std::time::Duration;
use wasm_bindgen_test::*;

#[component]
fn LoadSportConfigurationWrapper() -> impl IntoView {
    // requires Router context
    provide_context(ObjectListContext::<SportConfig, SportConfigIdQuery>::new());

    view! { <LoadSportConfiguration /> }
}

#[wasm_bindgen_test]
async fn test_new_sport_config() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set initial URL for creating a new sport config
    set_url(&format!(
        "/wasm_testing/new?sport_id={}",
        ts.generic_sport_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_context(core.clone());
        provide_global_context();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/wasm_testing/new") view=LoadSportConfigurationWrapper />
                </Routes>
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
        "/wasm_testing/edit?sport_id={}&sport_config_id={}",
        ts.generic_sport_id, ts.generic_sport_config_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_context(core.clone());
        provide_global_context();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/wasm_testing/edit") view=LoadSportConfigurationWrapper />
                </Routes>
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
}

#[wasm_bindgen_test]
async fn test_save_as_new_sport_config() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set URL with sport_id
    set_url(&format!(
        "/wasm_testing/edit?sport_id={}&sport_config_id={}",
        ts.generic_sport_id, ts.generic_sport_config_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_context(core.clone());
        provide_global_context();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/wasm_testing/edit") view=LoadSportConfigurationWrapper />
                </Routes>
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    web_sys::console::log_1(&"Test is starting".into());
    // verify that the form is populated with existing data
    let name_input = get_element_by_test_id("input-name")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    assert_eq!(name_input.value(), "Test Config 1");

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
    assert_eq!(cloned_configs[0].get_version().unwrap(), 0);
}
