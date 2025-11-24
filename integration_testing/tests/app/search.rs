use crate::common::{get_element_by_test_id, init_test_state};
use app::postal_addresses::{AddressParams, SearchPostalAddressInner};
use gloo_timers::future::sleep;
use isocountry::CountryCode;
use leptos::{
    mount::mount_to,
    prelude::*,
    tachys::dom::body,
    wasm_bindgen::JsCast,
    web_sys::{Event, HtmlInputElement, KeyboardEvent, KeyboardEventInit},
};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_search_postal_address() {
    let ts = init_test_state();
    let (params, set_params) = signal(Ok(AddressParams { uuid: None }));

    let core = ts.core.clone();

    let _mount_guard = mount_to(body(), move || {
        provide_socket_context();
        provide_context(core.clone());
        view! {
            <Router>
                <SearchPostalAddressInner params=params />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    let input = get_element_by_test_id("search-input");
    assert_eq!(
        input.get_attribute("placeholder").unwrap(),
        "Enter name of address you are searching..."
    );
    // simulate input: cast to HtmlInputElement and set value
    let input_elem = input.dyn_into::<HtmlInputElement>().unwrap();

    for index in -1..=(ts.entries.len() as i32) {
        input_elem.set_value(&ts.name_base);
        let event = Event::new("input").unwrap();
        input_elem.dispatch_event(&event).unwrap();
        sleep(Duration::from_millis(50)).await;
        let suggest = get_element_by_test_id("search-suggest");
        for num in 1..=ts.entries.len() {
            assert!(
                suggest
                    .text_content()
                    .unwrap()
                    .contains(&format!("{}{}", ts.name_base, num))
            );
        }

        // simulate selection of first suggestion via keyboard events
        // 1. ArrowUp / ArrowDown Event
        let key = if index < 0 { "ArrowUp" } else { "ArrowDown" };
        let num_key_down = if index < 0 { index.abs() } else { index + 1 };
        let index = index.rem_euclid(ts.entries.len() as i32) as usize;
        let id = ts.entries[index];
        let init_down = KeyboardEventInit::new();
        init_down.set_key(key);
        init_down.set_code(key);
        init_down.set_bubbles(true);
        init_down.set_cancelable(true);
        let event_down =
            KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init_down).unwrap();
        for _ in 0..num_key_down {
            input_elem.dispatch_event(&event_down).unwrap();
        }

        sleep(Duration::from_millis(10)).await;

        // 2. Enter Event
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
            .split("postal-address/")
            .last()
            .unwrap()
            .to_string();
        assert_eq!(url_id, id.to_string());

        // set address
        set_params.set(Ok(AddressParams { uuid: Some(id) }));
        sleep(Duration::from_millis(10)).await;
        let preview_name = get_element_by_test_id("preview-name")
            .text_content()
            .unwrap();
        assert!(preview_name.contains(&format!("{}{}", ts.name_base, index + 1)));
        let preview_street = get_element_by_test_id("preview-street")
            .text_content()
            .unwrap();
        assert!(preview_street.contains(&ts.street));
        let preview_postal = get_element_by_test_id("preview-postal_code")
            .text_content()
            .unwrap();
        assert!(preview_postal.contains(&ts.postal));
        let preview_locality = get_element_by_test_id("preview-locality")
            .text_content()
            .unwrap();
        assert!(preview_locality.contains(&ts.city));
        let preview_region = get_element_by_test_id("preview-region")
            .text_content()
            .unwrap();
        assert!(preview_region.contains(&ts.region));
        let preview_country = get_element_by_test_id("preview-country")
            .text_content()
            .unwrap();
        let expected_country_name = CountryCode::for_alpha2(&ts.country)
            .map(|c| c.name())
            .unwrap_or(&ts.country);
        assert!(preview_country.contains(expected_country_name));
        let preview_id = get_element_by_test_id("preview-id").text_content().unwrap();
        assert!(preview_id.contains(&id.to_string()));
        let preview_version = get_element_by_test_id("preview-version")
            .text_content()
            .unwrap();
        assert!(preview_version.contains("0"));
    }

    // test buttons
    let edit_button = get_element_by_test_id("btn-edit-address");
    assert_eq!(edit_button.text_content().unwrap(), "Edit");
    edit_button.click();
    sleep(Duration::from_millis(10)).await;
    let url_id = document()
        .location()
        .unwrap()
        .href()
        .unwrap()
        .split("postal-address/")
        .last()
        .unwrap()
        .split("/edit")
        .next()
        .unwrap()
        .to_string();
    let first_id = ts.entries[0];
    assert_eq!(url_id, first_id.to_string());

    let new_button = get_element_by_test_id("btn-new-address");
    assert_eq!(new_button.text_content().unwrap(), "New");
    new_button.click();
    sleep(Duration::from_millis(10)).await;
    let url = document().location().unwrap().href().unwrap();
    assert!(url.ends_with("/postal-address/new"));
}
