//!

use leptos::{prelude::*, wasm_bindgen::JsCast, web_sys::HtmlElement};

/// Blurs the currently active element.
///
/// This operation is deferred via `request_animation_frame` to ensure it runs
/// after pending DOM updates and event bubbling phases, which helps with
/// UI glitches like stuck dropdowns.
pub fn blur_active_element() {
    request_animation_frame(move || {
        if let Some(focused_element) = document().active_element() {
            let _ = focused_element
                .dyn_into::<HtmlElement>()
                .map(|el| el.blur());
        }
    });
}
