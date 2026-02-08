// common helpers for tests

use app_core::{Core, CoreBuilder, InitState, SportConfig, utils::traits::ObjectIdVersion};
use futures_util::lock::{Mutex, MutexGuard};
use generic_sport_plugin::GenericSportPlugin;
use generic_sport_plugin::config::GenericSportConfig;
use integration_testing::port_fakes::{FakeClientRegistryPort, FakeDatabasePort, make_addr};
use isocountry::CountryCode;
use leptos::{
    prelude::*,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{Event, HtmlElement, HtmlInputElement, HtmlSelectElement, window},
};
use sport_plugin_manager::SportPluginManagerMap;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

/// Global mutex to force serial execution of WASM tests sharing the DOM
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Helper function to acquire a lock for serial test execution.
/// This ensures that only one test manipulates the DOM at a time.
/// It creates a dedicated container for the app to avoid wiping out
/// the test runner's UI elements in the body.
///
/// Usage:
/// ```rust
/// let _guard = lock_test().await;
/// // ... rest of the test
/// ```
pub async fn lock_test() -> MutexGuard<'static, ()> {
    let mutex = TEST_LOCK.get_or_init(|| Mutex::new(()));
    let guard = mutex.lock().await;

    let doc = document();
    let body = doc.body().expect("document should have a body");

    // Remove existing test root if present (cleanup from previous test)
    if let Some(existing) = doc.get_element_by_id("test-app-root") {
        existing.remove();
    }

    // Create a new clean container for the app
    let app_root = doc.create_element("div").unwrap();
    app_root.set_id("test-app-root");
    body.append_child(&app_root).unwrap();

    guard
}

/// Helper function to get the dedicated test container.
pub fn get_test_root() -> HtmlElement {
    document()
        .get_element_by_id("test-app-root")
        .expect("test root not found - did you call lock_test()?")
        .dyn_into::<HtmlElement>()
        .unwrap()
}

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

/// Helper to simulate user typing and leaving the field (crucial for plugin fields)
pub fn set_input_value(test_id: &str, value: &str) {
    let input = get_element_by_test_id(test_id)
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    input.set_value(value);
    input.dispatch_event(&Event::new("input").unwrap()).unwrap();
    input
        .dispatch_event(&Event::new("change").unwrap())
        .unwrap();
    input.dispatch_event(&Event::new("blur").unwrap()).unwrap();
}

/// Helper to simulate user typing and leaving the field (crucial for plugin fields)
pub fn set_select_value(test_id: &str, value: &str) {
    let input = get_element_by_test_id(test_id)
        .dyn_into::<HtmlSelectElement>()
        .unwrap();
    input.set_value(value);
    input
        .dispatch_event(&Event::new("change").unwrap())
        .unwrap();
    input.dispatch_event(&Event::new("blur").unwrap()).unwrap();
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
    pub country: CountryCode,
    pub generic_sport_id: Uuid,
    pub generic_sport_config_id: Uuid,
}

pub fn init_test_state() -> InitialTestState {
    // All initialization logic is encapsulated here.
    let db = Arc::new(FakeDatabasePort::new());
    let cr = Arc::new(FakeClientRegistryPort::new());

    // Register Generic Sport Plugin
    let mut spm_map = SportPluginManagerMap::new();
    let generic_plugin = Arc::new(GenericSportPlugin::new());
    let generic_sport_id = generic_plugin.get_id_version().get_id();
    spm_map.register(generic_plugin).unwrap();
    let spm = Arc::new(spm_map);

    let core = CoreBuilder::new()
        .set_db(db.clone())
        .set_cr(cr.clone())
        .set_spm(spm.clone())
        .build();

    let core_arc = Arc::new(core);

    let name_base = "Test Address";
    let street = "123 Main St";
    let postal = "12345";
    let city = "Testcity";
    let region = "TS";
    let country = CountryCode::DEU;
    let mut entries = Vec::new();
    for index in 0..=2 {
        let name = format!("{name_base}{}", index + 1);
        let address = make_addr(&name, street, postal, city, region, country);
        let id = db.seed_postal_address(address);
        entries.push(id);
    }

    // Seed a config
    let generic_config = GenericSportConfig::default();
    let mut config = SportConfig::default();
    config
        .set_name("Test Config 1")
        .set_sport_id(generic_sport_id)
        .set_config(serde_json::to_value(&generic_config).unwrap());
    let generic_sport_config_id = db.seed_sport_config(config.clone());

    InitialTestState {
        core: core_arc,
        db,
        entries,
        name_base: name_base.into(),
        street: street.into(),
        postal: postal.into(),
        city: city.into(),
        region: region.into(),
        country,
        generic_sport_id,
        generic_sport_config_id,
    }
}
