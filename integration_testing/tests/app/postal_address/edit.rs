use crate::common::{get_element_by_test_id, init_test_state, set_url};
use app::{postal_addresses::PostalAddressForm, provide_global_state};
use app_core::DbpPostalAddress;
use gloo_timers::future::sleep;
use leptos::{
    mount::mount_to,
    prelude::*,
    tachys::dom::body,
    wasm_bindgen::JsCast,
    web_sys::{Event, HtmlInputElement, HtmlSelectElement},
};
use leptos_axum_socket::provide_socket_context;
use leptos_router::components::Router;
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_new_postal_address() {
    let ts = init_test_state();

    // 1. Set initial URL for creating a new address
    set_url("/postal-address/new_pa");

    let core = ts.core.clone();
    let _mount_guard = mount_to(body(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <PostalAddressForm />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    // create a new address by filling the form
    let name_input = get_element_by_test_id("input-name")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    name_input.set_value("New Name");
    let event = Event::new("input").unwrap();
    name_input.dispatch_event(&event).unwrap();

    let street_input = get_element_by_test_id("input-street")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    street_input.set_value(&ts.street);
    let event = Event::new("input").unwrap();
    street_input.dispatch_event(&event).unwrap();

    let postal_input = get_element_by_test_id("input-postal_code")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    postal_input.set_value(&ts.postal);
    let event = Event::new("input").unwrap();
    postal_input.dispatch_event(&event).unwrap();

    let city_input = get_element_by_test_id("input-locality")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    city_input.set_value(&ts.city);
    let event = Event::new("input").unwrap();
    city_input.dispatch_event(&event).unwrap();

    let region_input = get_element_by_test_id("input-region")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    region_input.set_value(&ts.region);
    let event = Event::new("input").unwrap();
    region_input.dispatch_event(&event).unwrap();

    let country_select = get_element_by_test_id("input-country")
        .dyn_into::<HtmlSelectElement>()
        .unwrap();
    country_select.set_value(&ts.country);
    let event = Event::new("change").unwrap();
    country_select.dispatch_event(&event).unwrap();

    sleep(Duration::from_millis(10)).await;
    let save_button = get_element_by_test_id("btn-save");
    save_button.click();

    sleep(Duration::from_millis(10)).await;

    let new_address = ts
        .db
        .list_postal_addresses(Some("New"), None)
        .await
        .unwrap();
    assert_eq!(new_address.len(), 1);
    assert_eq!(new_address[0].get_name(), "New Name");
}

#[wasm_bindgen_test]
async fn test_edit_postal_address() {
    let ts = init_test_state();

    // 1. Set initial URL for creating a new address
    let existing_id = ts.entries[0];
    set_url(&format!(
        "/postal-address/edit_pa?address_id={}",
        existing_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(body(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <PostalAddressForm />
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    // The component should react to the URL change.
    // A small delay helps ensure all reactive updates are processed.
    sleep(Duration::from_millis(10)).await;

    // verify that the form is populated with existing data
    let name_input = get_element_by_test_id("input-name")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    assert_eq!(name_input.value(), format!("{}1", ts.name_base));

    // modify some data and save
    let street_input = get_element_by_test_id("input-street")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    street_input.set_value("456 Another St");
    let event = Event::new("input").unwrap();
    street_input.dispatch_event(&event).unwrap();

    sleep(Duration::from_millis(10)).await;
    let save_button = get_element_by_test_id("btn-save");
    save_button.click();

    sleep(Duration::from_millis(10)).await;
    let updated_address = ts
        .db
        .get_postal_address(existing_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_address.get_street(), "456 Another St");
    assert_eq!(updated_address.get_version().unwrap(), 1);

    // now save existing address as new
    let name_input = get_element_by_test_id("input-name")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    name_input.set_value("Cloned Address");
    let event = Event::new("input").unwrap();
    name_input.dispatch_event(&event).unwrap();

    sleep(Duration::from_millis(10)).await;

    let save_as_new_button = get_element_by_test_id("btn-save-as-new");
    save_as_new_button.click();
    sleep(Duration::from_millis(10)).await;
    let cloned_addresses = ts
        .db
        .list_postal_addresses(Some("Cloned"), None)
        .await
        .unwrap();
    assert_eq!(cloned_addresses.len(), 1);
    assert_eq!(cloned_addresses[0].get_name(), "Cloned Address");
    assert_eq!(updated_address.get_street(), "456 Another St");
    assert_eq!(cloned_addresses[0].get_version().unwrap(), 0);
}
