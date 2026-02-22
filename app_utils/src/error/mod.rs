// app error

pub mod strategy;

use app_core::{CoreError, DbError, utils::validation::FieldError};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum AppError {
    /// update expects valid uuid
    #[error("Expected non nil id of object to update")]
    NilIdUpdate,

    /// Preserve inner server-fn error message/structure
    #[error(transparent)]
    ServerFn(#[from] ServerFnErrorErr),

    /// resource not found
    #[error("resource not found: {0} ({1})")]
    ResourceNotFound(String, Uuid),

    /// core error
    #[error("core error: {0}")]
    Core(#[from] CoreError),

    /// serde error
    #[error("serialization/deserialization error: {0}")]
    Serde(String),

    /// connection, pool, or other DB errors
    #[error("internal error: {0}")]
    Other(String),
}

// Let Leptos server functions know how to encode this error type
impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        // thanks to #[from], this is just:
        value.into()
    }
}

// serde_json::Error does not implement clone
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serde(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn map_db_unique_violation_to_field_error(
    err: &AppError,
    object_id: Uuid,
    field_name: &str,
) -> Option<FieldError> {
    if let AppError::Core(CoreError::Db(DbError::UniqueViolation(field_opt))) = err {
        let message = if let Some(field) = field_opt {
            format!("Unique constraint violation on field: {}", field)
        } else {
            "Unique constraint violation".to_string()
        };
        let field_error = FieldError::builder()
            .set_field(field_name)
            .add_message(message)
            .set_object_id(object_id)
            .build();

        return Some(field_error);
    }
    None
}
