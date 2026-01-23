use crate::common::{
    get_element_by_test_id, get_test_root, init_test_state, lock_test, set_input_value,
    set_select_value, set_url,
};
use app::{postal_addresses::PostalAddressForm, provide_global_state};
use app_core::DbpPostalAddress;
use gloo_timers::future::sleep;
use leptos::{mount::mount_to, prelude::*, wasm_bindgen::JsCast, web_sys::HtmlInputElement};
use leptos_axum_socket::provide_socket_context;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_new_postal_address() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set initial URL for creating a new address
    set_url("/postal-address/new_pa");

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/postal-address/new_pa") view=PostalAddressForm />
                </Routes>
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    // create a new address by filling the form
    set_input_value("input-name", "New Name");
    set_input_value("input-street", &ts.street);
    set_input_value("input-postal_code", &ts.postal);
    set_input_value("input-locality", &ts.city);
    set_input_value("input-region", &ts.region);
    set_select_value("select-country", &ts.country);

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
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set initial URL for creating a new address
    let existing_id = ts.entries[0];
    set_url(&format!(
        "/postal-address/edit_pa?address_id={}",
        existing_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/postal-address/edit_pa") view=PostalAddressForm />
                </Routes>
            </Router>
        }
    });

    // The component should react to the URL change.
    // A small delay helps ensure all reactive updates are processed.
    sleep(Duration::from_millis(10)).await;

    // verify that the form is populated with existing data
    let name_input = get_element_by_test_id("input-name")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    assert_eq!(name_input.value(), format!("{}1", ts.name_base));

    // modify some data and save
    set_input_value("input-street", "456 Another St");

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
}

#[wasm_bindgen_test]
async fn test_save_as_new_postal_address() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set initial URL for creating a new address
    let existing_id = ts.entries[0];
    set_url(&format!(
        "/postal-address/edit_pa?address_id={}",
        existing_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_socket_context();
        provide_context(core.clone());
        provide_global_state();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/postal-address/edit_pa") view=PostalAddressForm />
                </Routes>
            </Router>
        }
    });

    // The component should react to the URL change.
    // A small delay helps ensure all reactive updates are processed.
    sleep(Duration::from_millis(10)).await;

    // verify that the form is populated with existing data
    let name_input = get_element_by_test_id("input-name")
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    assert_eq!(name_input.value(), format!("{}1", ts.name_base));

    // now save existing address as new
    set_input_value("input-name", "Cloned Address");

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
    assert_eq!(cloned_addresses[0].get_version().unwrap(), 0);
}
