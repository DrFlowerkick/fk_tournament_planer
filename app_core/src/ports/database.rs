// database port

use crate::{PostalAddress, SportConfig, Stage, TournamentBase};
use async_trait::async_trait;
use isocountry::CountryCodeParseErr;
use serde::{Deserialize, Serialize};
use std::any::Any;
use thiserror::Error;
use uuid::Uuid;

/// database port trait
#[async_trait]
pub trait DatabasePort:
    DbpPostalAddress + DbpSportConfig + DbpTournamentBase + DbpStage + Any
{
    async fn ping_db(&self) -> DbResult<()>;
}

/// database port trait for postal address
#[async_trait]
pub trait DbpPostalAddress: Send + Sync {
    async fn get_postal_address(&self, id: Uuid) -> DbResult<Option<PostalAddress>>;
    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress>;
    async fn list_postal_addresses(
        &self,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<PostalAddress>>;
}

/// database port trait for sport config
#[async_trait]
pub trait DbpSportConfig: Send + Sync {
    async fn get_sport_config(&self, config_id: Uuid) -> DbResult<Option<SportConfig>>;
    async fn save_sport_config(&self, sport_config: &SportConfig) -> DbResult<SportConfig>;
    async fn list_sport_configs(
        &self,
        sport_id: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<SportConfig>>;
}
/// database port trait for tournament base
#[async_trait]
pub trait DbpTournamentBase: Send + Sync {
    async fn get_tournament_base(&self, base_id: Uuid) -> DbResult<Option<TournamentBase>>;
    async fn save_tournament_base(
        &self,
        tournament_base: &TournamentBase,
    ) -> DbResult<TournamentBase>;
    async fn list_tournament_bases(
        &self,
        sport_id: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<TournamentBase>>;
}

/// database port trait for stage
#[async_trait]
pub trait DbpStage: Send + Sync {
    async fn get_stage_by_id(&self, stage_id: Uuid) -> DbResult<Option<Stage>>;
    async fn get_stage_by_number(
        &self,
        tournament_base_id: Uuid,
        number: u32,
    ) -> DbResult<Option<Stage>>;
    async fn save_stage(&self, stage: &Stage) -> DbResult<Stage>;
    async fn list_stages_of_tournament(&self, tournament_id: Uuid) -> DbResult<Vec<Stage>>;
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum DbError {
    /// row id is nil
    #[error("id of row is nil")]
    NilRowId,

    /// row version is negativ
    #[error("version of row is negativ")]
    NegativeRowVersion,

    /// row version is out of range
    #[error("version of row is out of range of u32")]
    RowVersionOutOfRange,

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

    /// transient DB problems (retry may work)
    #[error("serialization failure")]
    SerializationFailure,

    /// invalid country code
    #[error("invalid country code: {0}")]
    InvalidCountryCode(String),

    /// connection, pool, or other DB errors
    #[error("internal error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for DbError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Database Error converted to string: {:?}", err);
        Self::Other(err.to_string())
    }
}

impl From<CountryCodeParseErr> for DbError {
    fn from(err: CountryCodeParseErr) -> Self {
        tracing::error!("Country code parse error: {:?}", err);
        Self::InvalidCountryCode(err.to_string())
    }
}

pub type DbResult<T> = Result<T, DbError>;
