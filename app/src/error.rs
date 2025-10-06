// app error

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
use thiserror::Error;

use app_core::DbError;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum AppError {
    /// Preserve inner server-fn error message/structure
    #[error(transparent)]
    ServerFn(#[from] ServerFnErrorErr),

    /// Your own DB/domain errors (serialized as string over the wire)
    #[error("database error: {0}")]
    Db(String),
}

// Let Leptos server functions know how to encode this error type
impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        // thanks to #[from], this is just:
        value.into()
    }
}

// since anyhow does not support serde, wie have to convert DbError to string
impl From<DbError> for AppError {
    fn from(e: DbError) -> Self {
        AppError::Db(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
