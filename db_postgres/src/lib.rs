// diesel postgres implementation of database port

pub mod helpers;
pub mod postal_address;
pub mod schema;

pub use helpers::*;

use anyhow::{Context, Result, anyhow};
use app_core::{DatabasePort, DbError, DbResult};
use async_trait::async_trait;
use diesel::{dsl::sql, select, sql_types::Bool};
use diesel_async::{
    AsyncMigrationHarness, AsyncPgConnection, RunQueryDsl,
    pooled_connection::{
        AsyncDieselConnectionManager,
        bb8::{Pool, PooledConnection},
    },
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tracing::{info, instrument, warn};
use url::Url;

/// embed migrations
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct PgDb {
    pool: Pool<AsyncPgConnection>,
}

impl PgDb {
    pub async fn new(database: Url) -> Result<Self> {
        let config = AsyncDieselConnectionManager::new(database);
        Ok(PgDb {
            pool: Pool::builder().build(config).await?,
        })
    }
    #[instrument(name = "db.migration", skip(self))]
    pub async fn run_migration(&self) -> DbResult<()> {
        let conn = self
            .pool
            .get_owned()
            .await
            .map_err(|e| DbError::Other(e.into()))?;
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut harness = AsyncMigrationHarness::new(conn);
            harness
                .run_pending_migrations(MIGRATIONS)
                .map_err(|e| anyhow!("migration failed: {e}"))?;
            Ok(())
        })
        .await
        .context("Join error while running migrations")??;

        info!("Migrations applied successfully");
        Ok(())
    }
    #[instrument(name = "db.conn.get", skip(self))]
    pub async fn new_connection(&self) -> DbResult<PooledConnection<'_, AsyncPgConnection>> {
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

#[async_trait]
impl DatabasePort for PgDb {
    #[instrument(name = "db.ping", skip(self))]
    async fn ping_db(&self) -> DbResult<()> {
        let mut conn = self.new_connection().await?;
        select(sql::<Bool>("1=1"))
            .execute(&mut conn)
            .await
            .map_err(|e| DbError::Other(e.into()))?;
        Ok(())
    }
}

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
