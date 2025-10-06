// diesel postgres implementation of database port

pub mod postal_address;
pub mod schema;

use anyhow::{Context, Result};
use app_core::{DatabasePort, DbError, DbResult};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{
        AsyncDieselConnectionManager,
        bb8::{Pool, PooledConnection},
    },
};
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;

pub struct PgDb {
    pool: Pool<AsyncPgConnection>,
}

impl PgDb {
    pub async fn new() -> Result<Arc<Self>> {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set. Hint: did you run dotenv()?")?;
        let config = AsyncDieselConnectionManager::new(database_url);
        Ok(Arc::new(PgDb {
            pool: Pool::builder().build(config).await?,
        }))
    }
    async fn new_connection(&self) -> DbResult<PooledConnection<'_, AsyncPgConnection>> {
        self.pool.get().await.map_err(|e| DbError::Other(e.into()))
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
