use crate::common::{get_element_by_test_id, init_test_state, set_url};
use app::{global_state::GlobalState, sport_config::SportConfigPage};
use app_core::SportConfig;
use gloo_timers::future::sleep;
use leptos::{mount::mount_to, prelude::*, tachys::dom::body};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use reactive_stores::Store;
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_config_search_renders() {
    let ts = init_test_state();

    // Seed a config
    let config = SportConfig {
        sport_id: ts.generic_sport_id,
        name: "Test Config 1".to_string(),
        ..Default::default()
    };
    ts.db.seed_sport_config(config);

    // 1. Set URL with sport_id
    set_url(&format!("/sport-config?sport_id={}", ts.generic_sport_id));

    let core = ts.core.clone();
    let _mount_guard = mount_to(body(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_context(Store::new(GlobalState::default()));
        view! {
            <Router>
                <SportConfigPage />
            </Router>
        }
    });

    sleep(Duration::from_millis(200)).await;

    // 2. Check if search input exists
    let input = get_element_by_test_id("search-sport-config-input");
    assert!(input.is_connected());

    // 3. Check if seeded config is listed
    let row = get_element_by_test_id("config-row-Test Config 1");
    assert!(row.inner_text().contains("Test Config 1"));
}
