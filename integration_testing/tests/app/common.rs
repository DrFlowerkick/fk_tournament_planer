// common helpers for tests

use app_core::{Core, InitState};
use integration_testing::port_fakes::{FakeDatabasePort, make_addr, make_core_with_fakes};
use leptos::{prelude::*, wasm_bindgen::JsCast, web_sys::HtmlElement};
use std::sync::Arc;
use uuid::Uuid;

/// Helper function to get an element by its data-testid attribute.
pub fn get_element_by_test_id(id: &str) -> HtmlElement {
    let document = document();
    document
        .query_selector(&format!("[data-testid='{}']", id))
        .unwrap()
        .unwrap_or_else(|| panic!("Element with test-id '{}' not found", id))
        .dyn_into::<HtmlElement>()
        .unwrap()
}

// A struct to hold all initial test data.
pub struct InitialTestState {
    pub core: Arc<Core<InitState>>,
    pub db: Arc<FakeDatabasePort>,
    pub entries: Vec<Uuid>,
    pub name_base: String,
    pub street: String,
    pub postal: String,
    pub city: String,
    pub region: String,
    pub country: String,
}

pub fn init_test_state() -> InitialTestState {
    // All initialization logic is encapsulated here.
    let (core, mock_db, _, _) = make_core_with_fakes();
    let core_arc = Arc::new(core);

    let name_base = "Test Address";
    let street = "123 Main St";
    let postal = "12345";
    let city = "Testcity";
    let region = "TS";
    let country = "DE";
    let mut entries = Vec::new();
    for index in 0..=2 {
        let name = format!("{name_base}{}", index + 1);
        let address = make_addr(&name, street, postal, city, region, country);
        let id = mock_db.seed_postal_address(address);
        entries.push(id);
    }

    InitialTestState {
        core: core_arc,
        db: mock_db,
        entries,
        name_base: name_base.into(),
        street: street.into(),
        postal: postal.into(),
        city: city.into(),
        region: region.into(),
        country: country.to_uppercase(),
    }
}
