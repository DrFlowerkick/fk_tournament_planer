use crate::common::{
    get_element_by_test_id, get_sub_element_by_test_id, get_test_root, init_test_state, lock_test,
    set_url,
};
use app::{postal_addresses::ListPostalAddresses, provide_global_context};
use gloo_timers::future::sleep;
use leptos::{mount::mount_to, prelude::*, wasm_bindgen::JsCast, web_sys::HtmlAnchorElement};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
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
        provide_context(core.clone());
        provide_global_context();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/postal-address") view=ListPostalAddresses />
                </Routes>
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    // check preview
    let first_row_id = format!("postal-address-row-{}", ts.entries[0]);
    let preview = get_element_by_test_id(&first_row_id)
        .text_content()
        .unwrap();
    assert!(preview.contains("Test Address1"));

    // check preview data test id of first address and check if it contains correct data
    let preview_name = get_sub_element_by_test_id(&first_row_id, "preview-address-name")
        .text_content()
        .unwrap();
    assert!(preview_name.contains(&format!("{}{}", ts.name_base, 1)));
    let preview_street = get_sub_element_by_test_id(&first_row_id, "preview-street")
        .text_content()
        .unwrap();
    assert!(preview_street.contains(&ts.street));
    let preview_postal = get_sub_element_by_test_id(&first_row_id, "preview-postal_code")
        .text_content()
        .unwrap();
    assert!(preview_postal.contains(&ts.postal));
    let preview_locality = get_sub_element_by_test_id(&first_row_id, "preview-locality")
        .text_content()
        .unwrap();
    assert!(preview_locality.contains(&ts.city));
    let preview_region = get_sub_element_by_test_id(&first_row_id, "preview-region")
        .text_content()
        .unwrap();
    assert!(preview_region.contains(&ts.region));
    let preview_country = get_sub_element_by_test_id(&first_row_id, "preview-country")
        .text_content()
        .unwrap();
    let expected_country_name = format!("{} ({})", ts.country.name(), ts.country.alpha2());
    assert!(preview_country.contains(&expected_country_name));
    let preview_id = get_sub_element_by_test_id(&first_row_id, "preview-address-id")
        .text_content()
        .unwrap();
    assert!(preview_id.contains(&ts.entries[0].to_string()));
    let preview_version = get_sub_element_by_test_id(&first_row_id, "preview-address-version")
        .text_content()
        .unwrap();
    assert!(preview_version.contains("0"));

    // click table and check URL update
    let row = get_element_by_test_id(&format!("postal-address-row-{}", ts.entries[0]));
    row.click();
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
    assert_eq!(url_id, ts.entries[0].to_string());

    // test new button URL
    let new_button = get_element_by_test_id("action-btn-new")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = new_button.href();
    assert!(href.ends_with("new"));
    assert_eq!(
        new_button.text_content().unwrap(),
        "Create New Postal Address"
    );

    // test buttons which show after click on table row
    let edit_button = get_element_by_test_id("action-btn-edit")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = edit_button.href();
    assert!(href.ends_with(&format!("edit?address_id={}", ts.entries[0])));
    assert_eq!(edit_button.text_content().unwrap(), "Edit");

    let copy_button = get_element_by_test_id("action-btn-copy")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = copy_button.href();
    assert!(href.ends_with(&format!("copy?address_id={}", ts.entries[0])));
    assert_eq!(copy_button.text_content().unwrap(), "Copy");
}
