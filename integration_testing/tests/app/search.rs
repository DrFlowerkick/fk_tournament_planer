use crate::common::get_element_by_test_id;
use app::postal_addresses::{AddressParams, SearchPostalAddressInner};
use gloo_timers::future::sleep;
use integration_testing::port_fakes::{make_addr, make_core_with_fakes};
use leptos::{
    mount::mount_to,
    prelude::*,
    tachys::dom::body,
    wasm_bindgen::JsCast,
    web_sys::{Event, HtmlInputElement, KeyboardEvent, KeyboardEventInit},
};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_search_postal_address_basic() {
    let (params, set_params) = signal(Ok(AddressParams { uuid: None }));
    let (core, mock_db, _) = make_core_with_fakes();
    let core = Arc::new(core);

    let street = "123 Main St";
    let postal = "12345";
    let city = "Testcity";
    let region = "TS";
    let country = "Testland";
    let mut entries: Vec<(usize, Uuid)> = Vec::new();
    for index in 0..=2 {
        let name = format!("Test Address {}", index + 1);
        let address = make_addr(&name, street, postal, city, region, country);
        let id = mock_db.seed(address);
        entries.push((index + 1, id));
    }

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

    for (num_key_down, id) in entries {
        input_elem.set_value("Test");
        let event = Event::new("input").unwrap();
        input_elem.dispatch_event(&event).unwrap();
        sleep(Duration::from_millis(50)).await;
        let suggest = get_element_by_test_id("search-suggest");
        assert!(suggest.text_content().unwrap().contains("Test Address 1"));
        assert!(suggest.text_content().unwrap().contains("Test Address 2"));
        assert!(suggest.text_content().unwrap().contains("Test Address 3"));

        // simulate selection of first suggestion via keyboard events
        // 1. ArrowDown Event
        let init_down = KeyboardEventInit::new();
        init_down.set_key("ArrowDown");
        init_down.set_code("ArrowDown");
        init_down.set_bubbles(true);
        init_down.set_cancelable(true);
        let event_down = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init_down)
            .expect("Failed to create ArrowDown event");
        for _ in 0..num_key_down {
            input_elem
                .dispatch_event(&event_down)
                .expect("Failed to dispatch ArrowDown event");
        }

        sleep(Duration::from_millis(10)).await;

        // 2. Enter Event
        let init_enter = KeyboardEventInit::new();
        init_enter.set_key("Enter");
        init_enter.set_code("Enter");
        init_enter.set_bubbles(true);
        init_enter.set_cancelable(true);
        let event_enter = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init_enter)
            .expect("Failed to create Enter event");
        input_elem
            .dispatch_event(&event_enter)
            .expect("Failed to dispatch Enter event");

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
        assert!(preview_name.contains(&format!("Test Address {}", num_key_down)));
        let preview_street = get_element_by_test_id("preview-street")
            .text_content()
            .unwrap();
        assert!(preview_street.contains(street));
        let preview_postal = get_element_by_test_id("preview-postal_code")
            .text_content()
            .unwrap();
        assert!(preview_postal.contains(postal));
        let preview_locality = get_element_by_test_id("preview-locality")
            .text_content()
            .unwrap();
        assert!(preview_locality.contains(city));
        let preview_region = get_element_by_test_id("preview-region")
            .text_content()
            .unwrap();
        assert!(preview_region.contains(region));
        let preview_country = get_element_by_test_id("preview-country")
            .text_content()
            .unwrap();
        assert!(preview_country.contains(country.to_uppercase().as_str()));
        let preview_id = get_element_by_test_id("preview-id").text_content().unwrap();
        assert!(preview_id.contains(&id.to_string()));
        let preview_version = get_element_by_test_id("preview-version")
            .text_content()
            .unwrap();
        assert!(preview_version.contains("0"));
    }
}
