use crate::common::get_element_by_test_id;
use app::components::banner::{AcknowledgmentAndNavigateBanner, AcknowledgmentBanner};
use gloo_timers::future::sleep;
use leptos::{mount::mount_to, prelude::*, tachys::dom::body};
use leptos_router::components::Router;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_acknowledgment_banner_display_and_acknowledge() {
    let ack_called = Arc::new(RwLock::new(false));
    let ack_called_clone = ack_called.clone();
    let ack_action = move || {
        let mut ack_called = ack_called_clone.write().unwrap();
        *ack_called = true;
    };

    let _mount_guard = mount_to(body(), move || {
        view! {
            <AcknowledgmentBanner
                msg="Test Message"
                ack_btn_text="Acknowledge"
                ack_action=move || ack_action()
            />
        }
    });
    sleep(Duration::from_millis(10)).await;
    let msg = get_element_by_test_id("acknowledgment-banner")
        .text_content()
        .unwrap();
    assert!(msg.contains("Test Message"));
    let button = get_element_by_test_id("btn-acknowledgment-action");
    assert_eq!(button.text_content().unwrap(), "Acknowledge");
    button.click();
    assert!(*ack_called.read().unwrap());
}

#[wasm_bindgen_test]
async fn test_acknowledgment_and_navigate_banner_display_and_acknowledge() {
    let ack_called = Arc::new(RwLock::new(false));
    let ack_called_clone = ack_called.clone();
    let ack_action = move || {
        let mut ack_called = ack_called_clone.write().unwrap();
        *ack_called = true;
    };

    let _mount_guard = mount_to(body(), move || {
        view! {
            <Router>
                <AcknowledgmentAndNavigateBanner
                    msg="Test Message"
                    ack_btn_text="Acknowledge"
                    ack_action=move || ack_action()
                    nav_btn_text="Navigate"
                    navigate_url="/some-path".to_string()
                />
            </Router>
        }
    });
    sleep(Duration::from_millis(10)).await;
    let msg = get_element_by_test_id("acknowledgment-navigate-banner")
        .text_content()
        .unwrap();
    assert!(msg.contains("Test Message"));
    let button = get_element_by_test_id("btn-acknowledgment-navigate-action");
    assert_eq!(button.text_content().unwrap(), "Acknowledge");
    button.click();
    assert!(*ack_called.read().unwrap());

    let nav_button = get_element_by_test_id("btn-acknowledgment-navigate");
    assert_eq!(nav_button.text_content().unwrap(), "Navigate");
    // should not panic
    nav_button.click();
    // Note: Actual navigation testing would require more setup and is not performed here.
}
