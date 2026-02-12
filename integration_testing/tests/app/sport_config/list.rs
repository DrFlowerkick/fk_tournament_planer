use crate::common::{get_element_by_test_id, get_test_root, init_test_state, lock_test, set_url};
use app::{home::ListSportConfigurations, provide_global_context};
use gloo_timers::future::sleep;
use leptos::{mount::mount_to, prelude::*, wasm_bindgen::JsCast, web_sys::HtmlAnchorElement};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_config_search_renders() {
    // Acquire lock and clean DOM.
    let _guard = lock_test().await;

    let ts = init_test_state();

    // 1. Set URL with sport_id
    set_url(&format!(
        "/wasm_testing/sport?sport_id={}",
        ts.generic_sport_id
    ));

    let core = ts.core.clone();
    let _mount_guard = mount_to(get_test_root(), move || {
        provide_context(core.clone());
        provide_global_context();
        view! {
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/wasm_testing/sport") view=ListSportConfigurations />
                </Routes>
            </Router>
        }
    });

    sleep(Duration::from_millis(10)).await;

    // click table and check URL update
    let row = get_element_by_test_id(&format!("table-entry-row-{}", ts.generic_sport_config_id));
    row.click();
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

    // check preview
    let preview = get_element_by_test_id("table-entry-detailed-preview")
        .text_content()
        .unwrap();
    assert!(preview.contains("~30 min"));

    // test new button URL
    let new_button = get_element_by_test_id("action-btn-new")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = new_button.href();
    assert!(href.ends_with(&format!("new?sport_id={}", ts.generic_sport_id)));
    assert_eq!(
        new_button.text_content().unwrap(),
        "Create New Configuration"
    );

    // test buttons which show after click on table row
    let edit_button = get_element_by_test_id("action-btn-edit")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = edit_button.href();
    assert!(href.ends_with(&format!(
        "edit?sport_id={}&sport_config_id={}",
        ts.generic_sport_id, ts.generic_sport_config_id
    )));
    assert_eq!(edit_button.text_content().unwrap(), "Edit");

    let copy_button = get_element_by_test_id("action-btn-copy")
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    let href = copy_button.href();
    assert!(href.ends_with(&format!(
        "copy?sport_id={}&sport_config_id={}",
        ts.generic_sport_id, ts.generic_sport_config_id
    )));
    assert_eq!(copy_button.text_content().unwrap(), "Copy");
}
