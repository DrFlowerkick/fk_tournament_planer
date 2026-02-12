//! Definitions for error types used throughout core.

use crate::{
    CrError, DbError, SportError,
    utils::validation::{FieldError, ValidationErrors},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum CoreError {
    /// database error
    #[error("database error: {0}")]
    Db(#[from] DbError),

    /// client registry error
    #[error("client registry error: {0}")]
    Cr(#[from] CrError),

    /// sport error
    #[error("sport error: {0}")]
    Sport(#[from] SportError),

    /// Generic validation error of one field of an entity
    /// Returns the first error only
    #[error("field validation error: {0}")]
    Field(#[from] FieldError),

    /// Generic collected validation errors holding stringified field names.
    /// This works for ANY entity (PostalAddress, SportConfig, etc.)
    #[error("validation error: {0:?}")]
    Validation(#[from] ValidationErrors),

    /// Missing ID where one is required
    #[error("missing ID of: {0}")]
    MissingId(String),

    /// Parsing error for enums of core
    #[error("parsing error: {0}")]
    ParsingError(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    pub fn is_optimistic_lock_conflict(&self) -> bool {
        matches!(self, CoreError::Db(DbError::OptimisticLockConflict))
    }
    pub fn is_unique_violation(&self) -> bool {
        matches!(self, CoreError::Db(DbError::UniqueViolation(_)))
    }
    pub fn get_field_error(&self) -> Option<&FieldError> {
        if let CoreError::Field(field_error) = self {
            Some(field_error)
        } else {
            None
        }
    }
}
