//!

use leptos::{prelude::*, wasm_bindgen::JsCast, web_sys::HtmlElement};

pub fn blur_active_element() {
    if let Some(focused_element) = document().active_element() {
        let _ = focused_element
            .dyn_into::<HtmlElement>()
            .map(|el| el.blur());
    }
}
