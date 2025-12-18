use crate::common::{get_element_by_test_id, get_test_root, init_test_state, lock_test, set_url};
use app::{provide_global_state, sport_config::SearchSportConfig};
use gloo_timers::future::sleep;
use leptos::{
    mount::mount_to,
    prelude::*,
    wasm_bindgen::JsCast,
    web_sys::{Event, HtmlAnchorElement, HtmlInputElement, KeyboardEvent, KeyboardEventInit},
};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_config_search_renders() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set URL with sport_id
    set_url(&format!("/sport?sport_id={}", ts.generic_sport_id));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <SearchSportConfig />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    let input = get_element_by_test_id("sport_config_id-search-input");
    assert_eq!(
        input.get_attribute("placeholder").unwrap(),
        "Enter name of sport configuration you are searching..."
    );
    // simulate input: cast to HtmlInputElement, set value, and check suggestions
    let input_elem = input.dyn_into::<HtmlInputElement>().unwrap();

    input_elem.set_value(&"Test");
    let event = Event::new("input").unwrap();
    input_elem.dispatch_event(&event).unwrap();
    sleep(Duration::from_millis(50)).await;
    let suggest = get_element_by_test_id("sport_config_id-search-suggest");
    assert!(suggest.text_content().unwrap().contains("Test Config 1"));

    // Select first sport configuration: press ArrowDown and Enter
    let init_down = KeyboardEventInit::new();
    init_down.set_key("ArrowDown");
    init_down.set_code("ArrowDown");
    init_down.set_bubbles(true);
    init_down.set_cancelable(true);
    let event_down =
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init_down).unwrap();
    input_elem.dispatch_event(&event_down).unwrap();
    sleep(Duration::from_millis(10)).await;

    let init_enter = KeyboardEventInit::new();
    init_enter.set_key("Enter");
    init_enter.set_code("Enter");
    init_enter.set_bubbles(true);
    init_enter.set_cancelable(true);
    let event_enter =
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init_enter).unwrap();
    input_elem.dispatch_event(&event_enter).unwrap();
    sleep(Duration::from_millis(10)).await;

    let url_id = document()
        .location()
        .unwrap()
        .href()
        .unwrap()
        .split("sport_config_id=")
        .last()
        .unwrap()
        .to_string();
    assert_eq!(url_id, ts.generic_sport_config_id.to_string());
    assert_eq!(input_elem.value(), "Test Config 1");

    // check preview
    let preview = get_element_by_test_id("sport-config-preview")
        .text_content()
        .unwrap();
    assert!(preview.contains("Test Config 1"));
    assert!(preview.contains("Expected Match Duration: 30 minutes"));

    // test buttons
    let edit_button = get_element_by_test_id("btn-edit-sport-config")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = edit_button.href();
    println!("Edit-Button href: {}", href);
    assert!(href.ends_with(&format!(
        "edit_sc?sport_id={}&sport_config_id={}",
        ts.generic_sport_id, ts.generic_sport_config_id
    )));

    let new_button = get_element_by_test_id("btn-new-sport-config")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = new_button.href();
    assert!(href.ends_with(&format!(
        "new_sc?sport_id={}&sport_config_id={}",
        ts.generic_sport_id, ts.generic_sport_config_id
    )));
    assert_eq!(new_button.text_content().unwrap(), "New");
}
