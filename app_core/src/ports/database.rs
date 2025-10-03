// database port

use crate::PostalAddress;
use async_trait::async_trait;
use uuid::Uuid;
use thiserror::Error;

/// database port trait
pub trait DatabasePort: DbpPostalAddress {}

/// database port trait for postal address
#[async_trait]
pub trait DbpPostalAddress: Send + Sync {
    async fn get_postal_address(&self, id: Uuid) -> DbResult<Option<PostalAddress>>;
    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress>;
    async fn list_postal_addresses(&self, name_filter: Option<&str>, limit: Option<usize>) -> DbResult<Vec<PostalAddress>>;
}

#[derive(Debug, Error)]
pub enum DbError {
    /// Update could ot find matching id + version
    #[error("optimistic lock conflict")]
    OptimisticLockConflict,

    /// ID does not exist
    #[error("entity not found")]
    NotFound,

    /// constraint name if it is returned from db
    #[error("unique violation{0:?}")]
    UniqueViolation(Option<String>), 

    /// constraint name if it is returned from db
    #[error("foreign key violation{0:?}")]
    ForeignKeyViolation(Option<String>),

    /// constraint name if it is returned from db
    #[error("check violation{0:?}")]
    CheckViolation(Option<String>),

    // transient DB problems (retry may work)
    #[error("serialization failure")]
    SerializationFailure,

    // connection, pool, or other DB errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type DbResult<T> = Result<T, DbError>;
