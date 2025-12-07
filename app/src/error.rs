// app error

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
use std::fmt::Display;
use thiserror::Error;

use app_core::{DbError, SportError, utils::validation::ValidationErrors};

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum AppError {
    /// update expects valid uuid
    #[error("Expected non nil id of object to update")]
    NilIdUpdate,

    /// Preserve inner server-fn error message/structure
    #[error(transparent)]
    ServerFn(#[from] ServerFnErrorErr),

    /// validation error
    #[error("validation error: {0}")]
    ValidationErrors(String),

    /// Your own DB/domain errors (serialized as string over the wire)
    #[error("database error: {0}")]
    Db(String),

    /// sport error
    #[error("sport error: {0}")]
    Sport(String),

    /// generic error
    #[error("generic error: {0}")]
    Generic(String),
}

// Let Leptos server functions know how to encode this error type
impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        // thanks to #[from], this is just:
        value.into()
    }
}

// since anyhow does not support serde, we have to convert DbError to string
impl From<DbError> for AppError {
    fn from(e: DbError) -> Self {
        AppError::Db(e.to_string())
    }
}

// since SportError does not support serde, we have to convert SportError to string
impl From<SportError> for AppError {
    fn from(e: SportError) -> Self {
        AppError::Sport(e.to_string())
    }
}

// since ValidationErrors does not support serde, we have to convert ValidationErrors to string
impl<F: Display> From<ValidationErrors<F>> for AppError {
    fn from(e: ValidationErrors<F>) -> Self {
        AppError::ValidationErrors(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
