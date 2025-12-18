use crate::common::{get_element_by_test_id, get_test_root, init_test_state, lock_test, set_url};
use app::{postal_addresses::SearchPostalAddress, provide_global_state};
use gloo_timers::future::sleep;
use isocountry::CountryCode;
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
async fn test_search_postal_address() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set initial URL for searching addresses
    set_url("/postal-address");

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <SearchPostalAddress />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    let input = get_element_by_test_id("address_id-search-input");
    assert_eq!(
        input.get_attribute("placeholder").unwrap(),
        "Enter name of address you are searching..."
    );
    // simulate input: cast to HtmlInputElement, set value, and check suggestions
    let input_elem = input.dyn_into::<HtmlInputElement>().unwrap();

    input_elem.set_value(&ts.name_base);
    let event = Event::new("input").unwrap();
    input_elem.dispatch_event(&event).unwrap();
    sleep(Duration::from_millis(50)).await;
    let suggest = get_element_by_test_id("address_id-search-suggest");
    for num in 1..=ts.entries.len() {
        assert!(
            suggest
                .text_content()
                .unwrap()
                .contains(&format!("{}{}", ts.name_base, num))
        );
    }

    // Select first address: press ArrowDown and Enter
    let id = ts.entries[0];
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
        .split("address_id=")
        .last()
        .unwrap()
        .to_string();
    assert_eq!(url_id, id.to_string());
    assert_eq!(input_elem.value(), format!("{}{}", ts.name_base, 1));

    // check preview
    let preview_name = get_element_by_test_id("preview-address-name")
        .text_content()
        .unwrap();
    assert!(preview_name.contains(&format!("{}{}", ts.name_base, 1)));
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
    let preview_id = get_element_by_test_id("preview-address-id")
        .text_content()
        .unwrap();
    assert!(preview_id.contains(&id.to_string()));
    let preview_version = get_element_by_test_id("preview-address-version")
        .text_content()
        .unwrap();
    assert!(preview_version.contains("0"));

    // test buttons
    let edit_button = get_element_by_test_id("btn-edit-address")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = edit_button.href();
    println!("Edit-Button href: {}", href);
    assert!(href.ends_with(&format!("edit_pa?address_id={}", ts.entries[0])));

    let new_button = get_element_by_test_id("btn-new-address")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = new_button.href();
    assert!(href.ends_with(&format!("new_pa?address_id={}", ts.entries[0])));
    assert_eq!(new_button.text_content().unwrap(), "New");
}
