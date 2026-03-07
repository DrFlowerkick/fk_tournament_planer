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

impl AppError {
    pub fn to_web_ui_msg(&self) -> String {
        match self {
            // 1. Optimistic Lock Conflict
            // The client registry and auto saving ensures, that always the latest version is loaded. If a version mismatch
            // occurs during saving, it means that parallel editing is happening. In this case, we still reload "automatically"
            // the current version. Therefore a manual reload by the user is not necessary
            // We inform the user about the parallel editing via a toast.
            // This should not happen often.
            AppError::Core(CoreError::Db(DbError::OptimisticLockConflict)) => {
                format!("Parallel editing conflict: {self}")
            }

            // 2. Unique Violation
            // Validation error: Input needs correction (e.g. "Name already taken").
            AppError::Core(CoreError::Db(DbError::UniqueViolation(field_opt))) => field_opt
                .as_ref()
                .map(|f| format!("A unique value is already in use: '{f}'."))
                .unwrap_or_else(|| "A unique value is already in use.".to_string()),

            // 3. Check Violation
            // Validation error: Database constraint failed (e.g. "age >= 0").
            // Treated like UniqueViolation: The user must correct the input.
            AppError::Core(CoreError::Db(DbError::CheckViolation(constraint_opt))) => {
                constraint_opt
                    .as_ref()
                    .map(|c| format!("Data validation failed (Constraint: {}).", c))
                    .unwrap_or_else(|| "Data validation failed.".to_string())
            }

            // Case 4 Specific Entity not found
            AppError::ResourceNotFound(entity, id) => {
                format!("'{entity}' with ID '{id}' could not be found.")
            }

            // Case 5: Generic Database Not Found
            AppError::Core(CoreError::Db(DbError::NotFound)) => {
                "The requested data could not be found in database.".to_string()
            }

            // 6. Everything else
            _ => {
                // "Fire & Forget" Toast
                // AppError implements Display via thiserror, so self.to_string() works fine.
                self.to_string()
            }
        }
    }

    pub fn to_field_error(&self, object_id: Uuid, field_name: &str) -> Option<FieldError> {
        if let AppError::Core(CoreError::Db(DbError::UniqueViolation(field_opt))) = self {
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
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Deserialize, Serialize, Error)]
#[error("Component error: {app_error} (Component ID: {component_id})")]
pub struct ComponentError {
    pub component_id: Uuid,
    pub app_error: AppError,
}

impl ComponentError {
    pub fn new(component_id: Uuid, app_error: AppError) -> Self {
        Self {
            component_id,
            app_error,
        }
    }
}

pub type ComponentResult<T> = Result<T, ComponentError>;
