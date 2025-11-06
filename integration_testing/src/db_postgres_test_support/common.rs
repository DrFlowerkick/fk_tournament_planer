//! Shared test utilities for DB adapter tests.
//!
//! Responsibilities:
//! - One-time tracing initialization for all tests.
//! - Per-test ephemeral database creation (UUID name) and migration.
//! - Cleanup of stale test databases (UUID-named) once at the beginning of the test run.
//! - Construction of a bb8 pool for diesel-async.
//!
//! Environment variables:
//! - POSTGRES_URL: base URL without a database name, e.g. "postgres://user:pass@localhost:5432"
//! - TEST_DB_PREFIX (optional): prefix for test DB names, default "tst_".
//! - DB_POOL_MAX (optional): e.g. "5"; default 5.
//! - OP_TIMEOUT_MS (optional): operation timeout configuration you might pass to your adapter (not used here directly).
//!
//! Important:
//! Call `init_tracing()` at the start of each test. Use `TestDb::new().await` to get a fresh DB + pool.

use db_postgres::{PgDb, url_custom_db};
use anyhow::Result;
use diesel::{QueryableByName, sql_query, sql_types::Text};
use diesel_async::{
    AsyncPgConnection, RunQueryDsl,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};
use std::{sync::Arc, sync::Once, time::Duration};
use tracing::{info, warn};
use url::Url;

static TRACING: Once = Once::new();
static BOOTSTRAP: Once = Once::new();

/// Initialize tracing an clear stale test db's once per test run. Call at the top of every test.
///
/// This uses `with_test_writer()` so output is properly captured by `cargo test`.
pub fn init_db_testing() {
    // Load .env first if present; ignore if missing (Docker sets envs)
    dotenvy::dotenv().ok();
    TRACING.call_once(|| {
        let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
            // Reasonable defaults for adapter tests; tune as needed.
            "info,db_postgres=debug,db_port=debug,diesel=warn".to_string()
        });

        let _ = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_test_writer()
            .try_init();
    });

    // Global one-time bootstrap that clears stale UUID-named test DBs.
    BOOTSTRAP.call_once(|| {
        // Fire-and-forget task; runs at first `init_db_testing()` call.
        // We block on it shortly to avoid interleaving with first DB creation.
        let fut = async {
            if let Err(e) = clear_stale_test_databases().await {
                warn!(error = ?e, "Failed to clear stale test databases");
            }
        };
        // Block briefly to avoid races with first DB create; keep it bounded.
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = tokio::time::timeout(Duration::from_secs(10), fut).await;
            });
        });
    });
}

/// Try to detect whether a DB name looks like one of our ephemeral test DBs.
/// Strategy:
/// - Optional prefix `TEST_DB_PREFIX` (default "tst_") must match.
/// - The remainder must parse as a UUID (hyphenated).
fn is_our_ephemeral_db(name: &str) -> bool {
    let prefix = std::env::var("TEST_DB_PREFIX").unwrap_or_else(|_| "tst_".into());
    if !name.starts_with(&prefix) {
        return false;
    }
    let rest = &name[prefix.len()..];
    uuid::Uuid::parse_str(rest).is_ok()
}

/// list database names of postgres server
#[derive(Debug, QueryableByName)]
struct DbRow {
    #[diesel(sql_type = Text)]
    datname: String,
}

async fn list_dbs(conn: &mut diesel_async::AsyncPgConnection) -> anyhow::Result<Vec<String>> {
    let rows: Vec<DbRow> = sql_query("SELECT datname FROM pg_database WHERE datistemplate = false")
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(|r| r.datname).collect())
}

/// terminate active sessions to database
async fn terminate_backends_for(
    conn: &mut diesel_async::AsyncPgConnection,
    db_name: &str,
) -> anyhow::Result<()> {
    // Close all sessions to this DB
    let _rows_affected = sql_query(
        "SELECT pg_terminate_backend(pid) \
         FROM pg_stat_activity \
         WHERE datname = $1",
    )
    .bind::<Text, _>(db_name)
    .execute(conn)
    .await?;
    Ok(())
}

/// save quoting on client side
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('\"', "\"\""))
}

/// drop database
async fn drop_db_if_exists(
    conn: &mut diesel_async::AsyncPgConnection,
    db_name: &str,
) -> anyhow::Result<()> {
    let sql = format!("DROP DATABASE IF EXISTS {}", quote_ident(db_name));
    sql_query(sql).execute(conn).await?;
    Ok(())
}

/// Drops all UUID-named test databases lingering from previous runs.
/// We protect "postgres", "template0", "template1" by filtering.
async fn clear_stale_test_databases() -> Result<()> {
    let admin_url = url_custom_db("postgres")?;
    let config = AsyncDieselConnectionManager::new(admin_url);
    let pool = Pool::builder().build(config).await?;
    let mut conn = pool.get().await?;

    let dbs = list_dbs(&mut conn).await?;
    for name in dbs {
        if is_our_ephemeral_db(&name) {
            terminate_backends_for(&mut conn, &name).await?;
            drop_db_if_exists(&mut conn, &name).await?;
        }
    }

    Ok(())
}

/// Create a fresh test database with a UUID-based name.
/// Must be executed against a maintenance DB (e.g., "postgres"), not inside a transaction.
/// Uses server-side quote_ident() to safely quote the identifier.
pub async fn create_test_database(conn: &mut AsyncPgConnection, db_name: &str) -> Result<()> {
    let sql = format!("CREATE DATABASE {}", quote_ident(db_name));
    sql_query(sql).execute(conn).await?;
    Ok(())
}

/// Creates a fresh ephemeral test database, runs migrations, and constructs a db adapter.
///
/// Typical usage:
/// ```ignore
/// let tdb = TestDb::new().await?;
/// ```
pub struct TestDb {
    db_name: String,
    db_url: Url,
    db: Arc<PgDb>,
}

impl TestDb {
    /// Create a new per-test database and initialize a connection pool.
    pub async fn new() -> Result<Self> {
        let admin_url = url_custom_db("postgres")?;
        let config = AsyncDieselConnectionManager::new(admin_url);
        let pool = Pool::builder().build(config).await?;
        let mut conn = pool.get().await?;

        // Name the test DB
        let prefix = std::env::var("TEST_DB_PREFIX").unwrap_or_else(|_| "tst_".into());
        let db_name = format!("{}{}", prefix, uuid::Uuid::new_v4());

        // Connect to admin and create the test DB
        create_test_database(&mut conn, &db_name).await?;

        let db_url = url_custom_db(&db_name)?;
        info!(%db_name, %db_url, "Created test database");

        // Connect to new test database
        let db = PgDb::new(db_url.clone()).await?;

        // Run migrations (blocking, but offloaded) against the new DB
        db.run_migration().await?;

        Ok(Self {
            db_name,
            db_url,
            db: Arc::new(db),
        })
    }

    /// The unique database name.
    pub fn database_name(&self) -> &str {
        &self.db_name
    }

    /// Full DATABASE_URL for this ephemeral DB.
    pub fn database_url(&self) -> &str {
        self.db_url.as_str()
    }

    /// adapter of implemented database port
    pub fn adapter(&self) -> Arc<PgDb> {
        self.db.clone()
    }
}
