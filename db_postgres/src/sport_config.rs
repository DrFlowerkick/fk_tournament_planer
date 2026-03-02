// implementation of sport config port

use crate::{
    PgDb, escape_like, map_db_err,
    schema::{sport_configs, sport_configs::dsl::*},
};
use app_core::{
    DbError, DbResult, DbpSportConfig, SportConfig,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::{
    dsl::sql,
    prelude::{
        AsChangeset, BoolExpressionMethods, ExpressionMethods, Insertable, OptionalExtension,
        PgSortExpressionMethods, QueryDsl, Queryable, TextExpressionMethods,
    },
    sql_types::BigInt,
};
use diesel_async::RunQueryDsl;
use serde_json::Value;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

// ------------------- DB-Row (SELECT/RETURNING) -------------------
#[derive(Debug, Queryable)]
pub struct DbSportConfig {
    pub id: Uuid,
    pub version: i64,
    pub sport_id: Uuid,
    pub name: String,
    pub config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Mapping DB -> Core
impl TryFrom<DbSportConfig> for SportConfig {
    type Error = DbError;

    fn try_from(r: DbSportConfig) -> Result<Self, Self::Error> {
        if r.id.is_nil() {
            return Err(DbError::NilRowId);
        }
        if r.version < 0 {
            return Err(DbError::NegativeRowVersion);
        }
        if r.version > u32::MAX as i64 {
            return Err(DbError::RowVersionOutOfRange);
        }
        let id_version = IdVersion::new(r.id, Some(r.version as u32));
        let mut sc = SportConfig::new(id_version);
        sc.set_sport_id(r.sport_id)
            .set_name(r.name)
            .set_config(r.config.clone());
        Ok(sc)
    }
}

// ------------------- INSERT / UPDATE -------------------
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = sport_configs)]
pub struct WriteDbSportConfig<'a> {
    pub sport_id: Uuid,
    pub name: &'a str,
    pub config: &'a Value,
}

// Mapping Core -> DB
impl<'a> From<&'a SportConfig> for WriteDbSportConfig<'a> {
    fn from(sc: &'a SportConfig) -> Self {
        WriteDbSportConfig {
            sport_id: sc.get_sport_id(),
            name: sc.get_name(),
            config: sc.get_config(),
        }
    }
}

// ------------------- Impl trait --------------------

#[async_trait]
impl DbpSportConfig for PgDb {
    #[instrument(name = "db.sc.get", skip(self), fields(id = %sc_id))]
    async fn get_sport_config(&self, sc_id: Uuid) -> DbResult<Option<SportConfig>> {
        let mut conn = self.new_connection().await?;
        let res = sport_configs
            .filter(id.eq(sc_id))
            .first::<DbSportConfig>(&mut conn)
            .await
            .optional()
            .map_err(map_db_err)?;

        match res {
            Some(res) => {
                let res = SportConfig::try_from(res)?;
                debug!("found_sport_config");
                Ok(Some(res))
            }
            None => {
                debug!("sport_config_not_found");
                Ok(None)
            }
        }
    }

    #[instrument(
        name = "db.sc.save",
        skip(self, sport_config),
        fields(
            id = ?sport_config.get_id(),
            version = sport_config.get_version(),
            is_new = sport_config.get_id_version().is_new()
        )
    )]
    async fn save_sport_config(&self, sport_config: &SportConfig) -> DbResult<SportConfig> {
        let mut conn = self.new_connection().await?;
        let w = WriteDbSportConfig::from(sport_config);

        if let IdVersion::Existing(inner) = sport_config.get_id_version() {
            // UPDATE with optimistic locking
            let res = diesel::update(
                sport_configs.filter(
                    id.eq(inner.get_id())
                        .and(version.eq(inner.get_version() as i64)),
                ),
            )
            .set((w, version.eq(sql::<BigInt>("version + 1"))))
            .returning((id, version, sport_id, name, config, created_at, updated_at))
            .get_result::<DbSportConfig>(&mut conn)
            .await;

            match res {
                Ok(row) => {
                    info!(saved_id = %row.id, new_version = row.version, "update_ok");
                    Ok(row.try_into()?)
                }
                Err(diesel::result::Error::NotFound) => {
                    let exists = diesel::select(diesel::dsl::exists(
                        sport_configs.filter(id.eq(inner.get_id())),
                    ))
                    .get_result::<bool>(&mut conn)
                    .await
                    .map_err(map_db_err)?;

                    if exists {
                        warn!("optimistic_lock_conflict");
                        Err(DbError::OptimisticLockConflict)
                    } else {
                        warn!("row_missing_on_update");
                        Err(DbError::NotFound)
                    }
                }
                Err(e) => {
                    error!(error = %e, "update_failed");
                    Err(map_db_err(e))
                }
            }
        } else {
            // INSERT
            let row = diesel::insert_into(sport_configs)
                .values(w)
                .returning((id, version, sport_id, name, config, created_at, updated_at))
                .get_result::<DbSportConfig>(&mut conn)
                .await
                .map_err(map_db_err)?;
            info!(saved_id = %row.id, "insert_ok");
            Ok(row.try_into()?)
        }
    }

    #[instrument(name = "db.sc.list", skip(self, name_filter, limit))]
    async fn list_sport_config_ids(
        &self,
        sport: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<Uuid>> {
        let mut conn = self.new_connection().await?;
        let mut query = sport_configs.into_boxed::<diesel::pg::Pg>();

        query = query.filter(sport_id.eq(sport));

        if let Some(f) = name_filter
            && !f.is_empty()
        {
            // Case-insensitive "contains" match; escape special chars for LIKE
            let pattern = format!("%{}%", escape_like(f));
            debug!("apply_name_filter");
            query = query.filter(name.like(pattern));
        }

        if let Some(lim) = limit {
            query = query.limit(lim as i64);
        }

        let rows = query
            .select(id)
            .order((name.asc().nulls_last(), created_at.asc()))
            .load::<Uuid>(&mut conn)
            .await
            .map_err(map_db_err)?;

        info!(count = rows.len(), "list_ok");
        Ok(rows)
    }
}
