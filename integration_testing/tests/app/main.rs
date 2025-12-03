#![cfg(feature = "test-mock")]

use wasm_bindgen_test::wasm_bindgen_test_configure;

// Configure wasm-pack-test to run in a browser for all tests in this crate
wasm_bindgen_test_configure!(run_in_browser);

mod banner;
mod common;
mod postal_address;
mod set_id_in_query_input_dropdown;
mod sport_config;
