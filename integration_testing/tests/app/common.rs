// common helpers for tests

use app_core::{Core, CoreBuilder, InitState, utils::id_version::VersionId};
use generic_sport_plugin::GenericSportPlugin;
use integration_testing::port_fakes::{FakeClientRegistryPort, FakeDatabasePort, make_addr};
use leptos::{
    prelude::*,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{HtmlElement, window},
};
use sport_plugin_manager::SportPluginManagerMap;
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

/// Helper function to set the browser URL for testing purposes.
pub fn set_url(path: &str) {
    let window = window().expect("no window");
    let history = window.history().expect("no history");
    history
        .push_state_with_url(&JsValue::NULL, "", Some(path))
        .expect("could not push state");
}

/// A struct to hold all initial test data.
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
    pub generic_sport_id: Uuid,
}

pub fn init_test_state() -> InitialTestState {
    // All initialization logic is encapsulated here.
    let db = Arc::new(FakeDatabasePort::new());
    let cr = Arc::new(FakeClientRegistryPort::new());

    // Register Generic Sport Plugin
    let mut spm_map = SportPluginManagerMap::new();
    let generic_plugin = Arc::new(GenericSportPlugin::new());
    let generic_sport_id = generic_plugin.get_id_version().get_id().unwrap();
    spm_map.register(generic_plugin).unwrap();
    let spm = Arc::new(spm_map);

    let core = CoreBuilder::new()
        .set_db(db.clone())
        .set_cr(cr.clone())
        .set_spm(spm.clone())
        .build();

    let core_arc = Arc::new(core);
    let mock_db = db; // alias to match previous code style if needed

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
        generic_sport_id,
    }
}
