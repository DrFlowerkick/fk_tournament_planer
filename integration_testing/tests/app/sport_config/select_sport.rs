use crate::common::{get_element_by_test_id, init_test_state, set_url};
use app::{global_state::GlobalState, sport_config::SportConfigPage};
use gloo_timers::future::sleep;
use leptos::{
    mount::mount_to,
    prelude::*,
    tachys::dom::body,
    wasm_bindgen::JsCast,
    web_sys::{Event, HtmlInputElement, KeyboardEvent, KeyboardEventInit},
};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use reactive_stores::Store;
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_plugin_selection_renders() {
    let ts = init_test_state();

    // Set initial URL
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

    let input = get_element_by_test_id("sport_id-search-input");
    assert_eq!(
        input.get_attribute("placeholder").unwrap(),
        "Enter name of sport you are searching..."
    );

    // simulate input: cast to HtmlInputElement, set value, and check suggestions
    let input_elem = input.dyn_into::<HtmlInputElement>().unwrap();

    input_elem.set_value("gen");
    let event = Event::new("input").unwrap();
    input_elem.dispatch_event(&event).unwrap();
    sleep(Duration::from_millis(50)).await;
    let suggest = get_element_by_test_id("sport_id-search-suggest");
    assert_eq!(suggest.text_content().unwrap(), "Generic Sport");

    // Select Generic Sport option: press ArrowDown and Enter
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

    // 5. Check URL and input value
    assert_eq!(input_elem.value(), "Generic Sport");
    let window = web_sys::window().unwrap();
    let location = window.location();
    let search = location.search().unwrap();
    assert!(search.contains(&format!("sport_id={}", ts.generic_sport_id)));
}
