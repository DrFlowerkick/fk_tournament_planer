// This is required for wasm-bindgen-test
#![cfg(test)]
use wasm_bindgen_test::*;

use app::banner::{AcknowledgmentAndNavigateBanner, AcknowledgmentBanner};
use gloo_timers::future::sleep;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
// Standard-Router-Komponenten. Nichts weiter.
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_location;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;
use leptos::web_sys::HtmlElement;
use leptos::wasm_bindgen::JsCast;

// Configure wasm-pack-test to run in a browser
wasm_bindgen_test_configure!(run_in_browser);

fn get_element_by_test_id(id: &str) -> HtmlElement {
    let document = leptos::prelude::document();
    document
        .query_selector(&format!("[data-testid='{}']", id))
        .unwrap()
        .unwrap_or_else(|| panic!("Element with test-id '{}' not found", id))
        .dyn_into::<HtmlElement>()
        .unwrap()
}



#[wasm_bindgen_test]
async fn test_acknowledgment_banner_display_and_acknowledge() {  
    let ack_called = Rc::new(Cell::new(false));
    let ack_called_clone = ack_called.clone();
    let ack_action = move || ack_called_clone.set(true);
    
    mount_to_body(move || {
        view! {
            <AcknowledgmentBanner
                msg="Test Message"
                ack_btn_text="Acknowledge"
                ack_action=move || ack_action()
            />
        }
    });
    sleep(Duration::from_millis(10)).await;
    let button = get_element_by_test_id("btn-acknowledgment-action");
    button.click();
    assert!(ack_called.get());
}

/*
#[wasm_bindgen_test]
async fn test_acknowledgment_and_navigate_banner_navigates() {
    #[component]
    fn LocationChecker() -> impl IntoView {
        let location = use_location();
        view! { <div data-testid="location-display">{move || location.pathname.get()}</div> }
    }

    // 1. Mount inside a standard <Router>.
    // Die `wasm-bindgen-test` Umgebung stellt eine funktionierende History-API bereit.
    let dispose = mount_to_body(move || {
        view! {
            <Router>
                <main>
                    <Routes>
                        <Route path="/start" view=move || view! {
                            <AcknowledgmentAndNavigateBanner
                                msg="Navigate Banner"
                                ack_btn_text="Ack"
                                ack_action=|| {}
                                nav_btn_text="Go Home"
                                navigate_url="/"
                            />
                            <LocationChecker/>
                        }/>
                        <Route path="/" view=LocationChecker/>
                    </Routes>
                </main>
            </Router>
        }
    });

    // Manuell zur Start-URL navigieren, um den Testzustand herzustellen.
    let navigate = use_navigate();
    navigate("/start", Default::default());

    sleep(Duration::from_millis(20)).await;

    // 2. Assert initial location
    let location_display = get_element_by_test_id("location-display");
    assert_eq!(location_display.text_content().unwrap(), "/start");

    // 3. Simulate click
    let nav_button = get_element_by_test_id("btn-acknowledgment-navigate");
    nav_button.click();

    sleep(Duration::from_millis(20)).await;

    // 4. Assert final location
    assert_eq!(location_display.text_content().unwrap(), "/");

    // 5. Cleanup
    dispose();
}
*/