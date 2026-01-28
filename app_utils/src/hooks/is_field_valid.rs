//! hook to check if a field is valid based on validation results

use app_core::utils::validation::{FieldResult, ValidationResult};
use leptos::prelude::*;
use uuid::Uuid;

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

pub fn is_object_field_valid<T: Send + Sync + 'static>(
    validation_result: Signal<ValidationResult<T>>,
    object_id: Signal<Option<Uuid>>,
    field: &str,
) -> FieldResult<()> {
    validation_result.with(|res| match res {
        Ok(_) => Ok(()),
        Err(err) => object_id.with(|maybe_o_id| {
            if let Some(o_id) = maybe_o_id
                && let Some(field_error) = err
                    .errors
                    .iter()
                    .find(|e| e.get_object_id() == *o_id && e.get_field() == field)
            {
                Err(field_error.clone())
            } else {
                Ok(())
            }
        }),
    })
}
