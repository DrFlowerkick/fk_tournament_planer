//! Hook to scroll an element into view when a signal changes

use leptos::html::H2;
use leptos::prelude::*;
use leptos::web_sys::{ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition};

/// Scrolls the referenced element into view when the component mounts.
///
/// # Arguments
/// * `node_ref` - The NodeRef attached to the container element of the page/component.
pub fn use_scroll_h2_into_view<T>(node_ref: NodeRef<H2>, trigger: Signal<T>)
where
    T: Sync + Send + 'static,
{
    Effect::new(move || {
        // track the trigger signal to re-run the effect when it changes
        trigger.track();
        // node_ref.get() returns None initially, but the Effect runs again
        // when the node is attached to the DOM.
        if let Some(element) = node_ref.get() {
            let options = ScrollIntoViewOptions::new();
            // Smooth scrolling looks nicer for nested updates, 'Auto' is instant jump
            options.set_behavior(ScrollBehavior::Smooth);
            // Align the top of the element with the top of the viewport
            options.set_block(ScrollLogicalPosition::Start);

            element.scroll_into_view_with_scroll_into_view_options(&options);
        }
    });
}
