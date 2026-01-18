//! implementation of tournament base port

use crate::{
    PgDb, escape_like, map_db_err,
    schema::{tournament_bases, tournament_bases::dsl::*},
};
use app_core::{
    DbError, DbResult, TournamentBase, TournamentMode, TournamentState, TournamentType,
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
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

// Wir benötigen diesen Trait wahrscheinlich in app_core/src/ports/database.rs definiert,
// hier nehme ich an, er existiert und hat die Signaturen wie in base.rs verwendet.
// Falls nicht, muss er in app_core definiert werden.
use app_core::DbpTournamentBase;

// ------------------- DB-Row (SELECT/RETURNING) -------------------
#[derive(Debug, Queryable)]
pub struct DbTournamentBase {
    pub id: Uuid,
    pub version: i64,
    pub name: String,
    pub sport_id: Uuid,
    pub num_entrants: i32,
    pub t_type: serde_json::Value,
    pub mode: serde_json::Value,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Mapping DB -> Core
impl TryFrom<DbTournamentBase> for TournamentBase {
    type Error = DbError;

    fn try_from(r: DbTournamentBase) -> Result<Self, Self::Error> {
        if r.id.is_nil() {
            return Err(DbError::NilRowId);
        }
        if r.version < 0 {
            return Err(DbError::NegativeRowVersion);
        }
        if r.version > u32::MAX as i64 {
            return Err(DbError::RowVersionOutOfRange);
        }

        let t_type_from_json: TournamentType = serde_json::from_value(r.t_type)
            .map_err(|e| DbError::Other(format!("Failed to deserialize t_type: {e}")))?;
        let mode_from_json: TournamentMode = serde_json::from_value(r.mode)
            .map_err(|e| DbError::Other(format!("Failed to deserialize mode: {e}")))?;
        let state_from_json: TournamentState = serde_json::from_value(r.state)
            .map_err(|e| DbError::Other(format!("Failed to deserialize state: {e}")))?;

        let id_version = IdVersion::new(r.id, Some(r.version as u32));
        let mut tb = TournamentBase::new(id_version);

        tb.set_name(r.name)
            .set_sport_id(r.sport_id)
            .set_num_entrants(r.num_entrants as u32)
            .set_tournament_type(t_type_from_json)
            .set_tournament_mode(mode_from_json)
            .set_tournament_state(state_from_json);

        Ok(tb)
    }
}

// ------------------- INSERT / UPDATE -------------------
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = tournament_bases)]
pub struct WriteDbTournamentBase<'a> {
    pub name: &'a str,
    pub sport_id: Uuid,
    pub num_entrants: i32,
    pub t_type: serde_json::Value,
    pub mode: serde_json::Value,
    pub state: serde_json::Value,
}

// Mapping Core -> DB
impl<'a> TryFrom<&'a TournamentBase> for WriteDbTournamentBase<'a> {
    type Error = DbError;

    fn try_from(tb: &'a TournamentBase) -> Result<Self, Self::Error> {
        Ok(WriteDbTournamentBase {
            name: tb.get_name(),
            sport_id: tb.get_sport_id(),
            num_entrants: tb.get_num_entrants() as i32,
            t_type: serde_json::to_value(tb.get_tournament_type())
                .map_err(|e| DbError::Other(format!("Failed to serialize t_type: {e}")))?,
            mode: serde_json::to_value(tb.get_tournament_mode())
                .map_err(|e| DbError::Other(format!("Failed to serialize mode: {e}")))?,
            state: serde_json::to_value(tb.get_tournament_state())
                .map_err(|e| DbError::Other(format!("Failed to serialize state: {e}")))?,
        })
    }
}

// ------------------- Impl trait --------------------

#[async_trait]
impl DbpTournamentBase for PgDb {
    #[instrument(name = "db.tb.get", skip(self), fields(id = %t_id))]
    async fn get_tournament_base(&self, t_id: Uuid) -> DbResult<Option<TournamentBase>> {
        let mut conn = self.new_connection().await?;
        let res = tournament_bases
            .filter(id.eq(t_id))
            .first::<DbTournamentBase>(&mut conn)
            .await
            .optional()
            .map_err(map_db_err)?;

        match res {
            Some(res) => {
                let res = TournamentBase::try_from(res)?;
                debug!("found_tournament_base");
                Ok(Some(res))
            }
            None => {
                debug!("tournament_base_not_found");
                Ok(None)
            }
        }
    }

    #[instrument(
        name = "db.tb.save",
        skip(self, tournament),
        fields(
            id = ?tournament.get_id(),
            version = tournament.get_version(),
            is_new = tournament.get_id().is_none()
        )
    )]
    async fn save_tournament_base(&self, tournament: &TournamentBase) -> DbResult<TournamentBase> {
        let mut conn = self.new_connection().await?;
        let w = WriteDbTournamentBase::try_from(tournament)?;

        match tournament.get_id_version() {
            // Case 1: UPDATE (Optimistic Locking)
            IdVersion::Existing(inner) => {
                let res = diesel::update(
                    tournament_bases.filter(
                        id.eq(inner.get_id())
                            .and(version.eq(inner.get_version() as i64)),
                    ),
                )
                .set((w, version.eq(sql::<BigInt>("version + 1"))))
                .returning((
                    id,
                    version,
                    name,
                    sport_id,
                    num_entrants,
                    t_type,
                    mode,
                    state,
                    created_at,
                    updated_at,
                ))
                .get_result::<DbTournamentBase>(&mut conn)
                .await;

                match res {
                    Ok(row) => {
                        info!(saved_id = %row.id, new_version = row.version, "update_ok");
                        Ok(row.try_into()?)
                    }
                    Err(diesel::result::Error::NotFound) => {
                        let exists = diesel::select(diesel::dsl::exists(
                            tournament_bases.filter(id.eq(inner.get_id())),
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
            }
            // Case 2: INSERT with specific ID (e.g. Migration)
            IdVersion::NewWithId(new_id) => {
                let row = diesel::insert_into(tournament_bases)
                    .values((id.eq(new_id), w))
                    .returning((
                        id,
                        version,
                        name,
                        sport_id,
                        num_entrants,
                        t_type,
                        mode,
                        state,
                        created_at,
                        updated_at,
                    ))
                    .get_result::<DbTournamentBase>(&mut conn)
                    .await
                    .map_err(map_db_err)?;

                info!(saved_id = %row.id, "insert_ok");
                Ok(row.try_into()?)
            }
            // Case 3: INSERT with DB-generated ID (Standard)
            IdVersion::New => {
                let row = diesel::insert_into(tournament_bases)
                    .values(w)
                    .returning((
                        id,
                        version,
                        name,
                        sport_id,
                        num_entrants,
                        t_type,
                        mode,
                        state,
                        created_at,
                        updated_at,
                    ))
                    .get_result::<DbTournamentBase>(&mut conn)
                    .await
                    .map_err(map_db_err)?;

                info!(saved_id = %row.id, "insert_ok");
                Ok(row.try_into()?)
            }
        }
    }

    #[instrument(name = "db.tb.list", skip(self, name_filter, limit))]
    async fn list_tournament_bases(
        &self,
        sport: Uuid,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<TournamentBase>> {
        let mut conn = self.new_connection().await?;
        let mut query = tournament_bases.into_boxed::<diesel::pg::Pg>();

        if !sport.is_nil() {
            query = query.filter(sport_id.eq(sport));
        }

        if let Some(f) = name_filter
            && !f.is_empty()
        {
            let pattern = format!("%{}%", escape_like(f));
            debug!("apply_name_filter");
            query = query.filter(name.like(pattern));
        }

        if let Some(lim) = limit {
            query = query.limit(lim as i64);
        }

        let rows = query
            .order((name.asc().nulls_last(), created_at.asc()))
            .load::<DbTournamentBase>(&mut conn)
            .await
            .map_err(map_db_err)?;

        info!(count = rows.len(), "list_ok");
        rows.into_iter().map(TournamentBase::try_from).collect()
    }
}
