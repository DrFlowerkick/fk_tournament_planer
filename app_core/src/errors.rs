//! Definitions for error types used throughout core.

use crate::{CrError, DbError, SportError, utils::validation::ValidationErrors};
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

    /// Generic validation error holding stringified field names.
    /// This works for ANY entity (PostalAddress, SportConfig, etc.)
    #[error("validation error: {0:?}")]
    Validation(#[from] ValidationErrors),
}

pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    pub fn is_optimistic_lock_conflict(&self) -> bool {
        matches!(self, CoreError::Db(DbError::OptimisticLockConflict))
    }
    pub fn is_unique_violation(&self) -> bool {
        matches!(self, CoreError::Db(DbError::UniqueViolation(_)))
    }
}
