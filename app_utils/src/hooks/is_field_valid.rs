//! hook to check if a field is valid based on validation results

use app_core::utils::validation::ValidationResult;
use leptos::prelude::*;

pub fn is_field_valid<T: Send + Sync + 'static>(
    validation_result: impl Fn() -> ValidationResult<T> + Sync + Send + 'static,
) -> Callback<&'static str, Option<String>> {
    Callback::new(move |field: &str| match validation_result() {
        Ok(_) => None,
        Err(err) => err.errors.iter().find(|e| e.get_field() == field).map(|e| {
            if e.get_message().is_empty() {
                e.to_string()
            } else {
                e.get_message().to_string()
            }
        }),
    })
}
