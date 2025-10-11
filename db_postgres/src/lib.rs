// diesel postgres implementation of database port

pub mod helpers;
pub mod postal_address;
pub mod schema;

pub use helpers::*;

use anyhow::{Context, Result};
use app_core::{DatabasePort, DbError, DbResult};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{
        AsyncDieselConnectionManager,
        bb8::{Pool, PooledConnection},
    },
};
use std::env;
use tracing::{instrument, warn};

pub struct PgDb {
    pool: Pool<AsyncPgConnection>,
}

impl PgDb {
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set. Hint: did you run dotenv()?")?;
        let config = AsyncDieselConnectionManager::new(database_url);
        Ok(PgDb {
            pool: Pool::builder().build(config).await?,
        })
    }
    #[instrument(name = "db.conn.get", skip(self))]
    async fn new_connection(&self) -> DbResult<PooledConnection<'_, AsyncPgConnection>> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => {
                // Pool exhausted or database unavailable
                warn!(error = %e, "pool_get_failed");
                Err(DbError::Other(e.into()))
            }
        }
    }
}

impl DatabasePort for PgDb {}

use diesel::result::{DatabaseErrorKind as K, Error as DE};

fn map_db_err(e: DE) -> DbError {
    match &e {
        DE::NotFound => DbError::NotFound,
        DE::DatabaseError(kind, info) => {
            let c = info.constraint_name().map(|s| s.to_string());
            match kind {
                K::UniqueViolation => DbError::UniqueViolation(c),
                K::ForeignKeyViolation => DbError::ForeignKeyViolation(c),
                K::CheckViolation => DbError::CheckViolation(c),
                K::SerializationFailure => DbError::SerializationFailure,
                _ => DbError::Other(anyhow::anyhow!(e)),
            }
        }
        _ => DbError::Other(anyhow::anyhow!(e)),
    }
}
