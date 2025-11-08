#![cfg(feature = "test-mock")]
use crate::common::get_element_by_test_id;
use app::postal_addresses::{AddressParams, SearchPostalAddressInner};
use gloo_timers::future::sleep;
use leptos::{mount::mount_to_body, prelude::*, wasm_bindgen::JsCast};
use leptos_router::components::Router;
use leptos_axum_socket::provide_socket_context;
use std::{sync::Arc, time::Duration};
use wasm_bindgen_test::*;
use integration_testing::port_fakes::{make_core_with_fakes, make_addr};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_search_postal_address_basic() {
    let params = Signal::derive(|| Ok(AddressParams { uuid: None }));
    let (core, mock_db, _) = make_core_with_fakes();
    let core = Arc::new(core);

    mount_to_body(move || {
        provide_socket_context();
        provide_context(core.clone());
        view! {
            <Router>
                <SearchPostalAddressInner params=params />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    for index in 0..2 {
        let name = format!("Test Address {}", index + 1);
        let street = "123 Main St";
        let postal = "12345";
        let city = "Testcity";
        let region = "TS";
        let country = "Testland";
        mock_db.seed(make_addr(&name, street, postal, city, region, country));
    }

    let input = get_element_by_test_id("search-input");
    assert_eq!(
        input.get_attribute("placeholder").unwrap(),
        "Enter name of address you are searching..."
    );
    // simulate input: cast to HtmlInputElement and set value
    let input_elem = input
        .dyn_into::<leptos::web_sys::HtmlInputElement>()
        .unwrap();
    input_elem.set_value("Test Address 1");
    let event = leptos::web_sys::Event::new("input").unwrap();
    input_elem.dispatch_event(&event).unwrap();
    sleep(Duration::from_millis(50)).await;
    let suggest = get_element_by_test_id("search-suggest");
    assert!(suggest.text_content().unwrap().contains("Test Address 1"));
}
