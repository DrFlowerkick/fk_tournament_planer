// common helpers for tests

use leptos::{prelude::*, wasm_bindgen::JsCast, web_sys::HtmlElement};

pub fn get_element_by_test_id(id: &str) -> HtmlElement {
    let document = document();
    document
        .query_selector(&format!("[data-testid='{}']", id))
        .unwrap()
        .unwrap_or_else(|| panic!("Element with test-id '{}' not found", id))
        .dyn_into::<HtmlElement>()
        .unwrap()
}
