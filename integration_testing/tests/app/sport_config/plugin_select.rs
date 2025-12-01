use crate::common::{get_element_by_test_id, init_test_state, set_url};
use app::{global_state::GlobalState, sport_config::SportConfigPage};
use gloo_timers::future::sleep;
use leptos::{
    mount::mount_to,
    prelude::*,
    tachys::dom::body,
    wasm_bindgen::JsCast,
    web_sys::{Event, HtmlSelectElement},
};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use reactive_stores::Store;
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_plugin_selection_renders() {
    let ts = init_test_state();

    // 1. Set initial URL
    set_url("/sport-config");

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

    sleep(Duration::from_millis(10)).await;

    // 2. Find select element
    let select = get_element_by_test_id("select-sport-plugin")
        .dyn_into::<HtmlSelectElement>()
        .expect("select-sport-plugin should be a select element");

    // 3. Check options
    let options = select.options();
    assert!(
        options.length() >= 2,
        "Should have at least 'Choose...' and one plugin"
    );

    let first_option = options
        .item(0)
        .unwrap()
        .dyn_into::<web_sys::HtmlOptionElement>()
        .unwrap();
    assert_eq!(first_option.text(), "Choose Sport...");

    // Find Generic Sport option
    let mut found_generic = false;
    let mut generic_value = String::new();

    for i in 1..options.length() {
        let opt = options
            .item(i)
            .unwrap()
            .dyn_into::<web_sys::HtmlOptionElement>()
            .unwrap();
        if opt.text().contains("Generic Sport") {
            found_generic = true;
            generic_value = opt.value();
            break;
        }
    }
    assert!(found_generic, "Generic Sport option not found");
    assert_eq!(generic_value, ts.generic_sport_id.to_string());

    // 4. Select Generic Sport
    select.set_value(&generic_value);
    let event = Event::new("change").unwrap();
    select.dispatch_event(&event).unwrap();

    sleep(Duration::from_millis(10)).await;

    // 5. Check URL
    let window = web_sys::window().unwrap();
    let location = window.location();
    let search = location.search().unwrap();
    assert!(search.contains(&format!("sport_id={}", ts.generic_sport_id)));
}
