//! hook to check if a field is valid based on validation results

use app_core::utils::validation::{FieldResult, ValidationResult};
use leptos::prelude::*;

pub fn is_field_valid<T: Send + Sync + 'static>(
    validation_result: Signal<ValidationResult<T>>,
    field: &str,
) -> FieldResult<()> {
    validation_result.with(|res| match res {
        Ok(_) => Ok(()),
        Err(err) => {
            if let Some(field_error) = err.errors.iter().find(|e| e.get_field() == field) {
                Err(field_error.clone())
            } else {
                Ok(())
            }
        }
    })
}
