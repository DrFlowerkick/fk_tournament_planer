//! implementation of stage port

use crate::{
    PgDb, map_db_err,
    schema::{stages, stages::dsl::*},
};
use app_core::{
    DbError, DbResult, DbpStage, Stage,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::{
    dsl::sql,
    prelude::{
        AsChangeset, BoolExpressionMethods, ExpressionMethods, Insertable, OptionalExtension,
        QueryDsl, Queryable,
    },
    sql_types::BigInt,
};
use diesel_async::RunQueryDsl;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

// ------------------- DB-Row (SELECT/RETURNING) -------------------
#[derive(Debug, Queryable)]
pub struct DbStage {
    pub id: Uuid,
    pub version: i64,
    pub tournament_id: Uuid,
    pub number: i32,
    pub num_groups: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Mapping DB -> Core
impl TryFrom<DbStage> for Stage {
    type Error = DbError;

    fn try_from(r: DbStage) -> Result<Self, Self::Error> {
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
        let mut s = Stage::new(id_version, r.number as u32);

        s.set_tournament_id(r.tournament_id)
            .set_num_groups(r.num_groups as u32);

        Ok(s)
    }
}

// ------------------- INSERT / UPDATE -------------------
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = stages)]
pub struct WriteDbStage {
    pub tournament_id: Uuid,
    pub number: i32,
    pub num_groups: i32,
}

// Mapping Core -> DB
impl<'a> TryFrom<&'a Stage> for WriteDbStage {
    type Error = DbError;

    fn try_from(s: &'a Stage) -> Result<Self, Self::Error> {
        Ok(WriteDbStage {
            tournament_id: s.get_tournament_id(),
            number: s.get_number() as i32,
            num_groups: s.get_num_groups() as i32,
        })
    }
}

// ------------------- Impl trait --------------------

#[async_trait]
impl DbpStage for PgDb {
    #[instrument(name = "db.stage.get_id", skip(self), fields(id = %stage_id))]
    async fn get_stage_by_id(&self, stage_id: Uuid) -> DbResult<Option<Stage>> {
        let mut conn = self.new_connection().await?;
        let res = stages
            .filter(id.eq(stage_id))
            .first::<DbStage>(&mut conn)
            .await
            .optional()
            .map_err(map_db_err)?;

        match res {
            Some(res) => {
                let res = Stage::try_from(res)?;
                debug!("found_stage");
                Ok(Some(res))
            }
            None => {
                debug!("stage_not_found");
                Ok(None)
            }
        }
    }

    #[instrument(name = "db.stage.get_num", skip(self), fields(tid = %t_id, num = %num))]
    async fn get_stage_by_number(&self, t_id: Uuid, num: u32) -> DbResult<Option<Stage>> {
        let mut conn = self.new_connection().await?;
        let res = stages
            .filter(tournament_id.eq(t_id))
            .filter(number.eq(num as i32))
            .first::<DbStage>(&mut conn)
            .await
            .optional()
            .map_err(map_db_err)?;

        match res {
            Some(res) => {
                let res = Stage::try_from(res)?;
                debug!("found_stage_by_number");
                Ok(Some(res))
            }
            None => {
                debug!("stage_by_number_not_found");
                Ok(None)
            }
        }
    }

    #[instrument(
        name = "db.stage.save",
        skip(self, stage),
        fields(
            id = ?stage.get_id(),
            version = stage.get_version(),
            is_new = stage.get_id().is_none()
        )
    )]
    async fn save_stage(&self, stage: &Stage) -> DbResult<Stage> {
        let mut conn = self.new_connection().await?;
        let w = WriteDbStage::try_from(stage)?;

        match stage.get_id_version() {
            // Case 1: UPDATE (Optimistic Locking)
            IdVersion::Existing(inner) => {
                let res = diesel::update(
                    stages.filter(
                        id.eq(inner.get_id())
                            .and(version.eq(inner.get_version() as i64)),
                    ),
                )
                .set((w, version.eq(sql::<BigInt>("version + 1"))))
                .returning((
                    id,
                    version,
                    tournament_id,
                    number,
                    num_groups,
                    created_at,
                    updated_at,
                ))
                .get_result::<DbStage>(&mut conn)
                .await;

                match res {
                    Ok(row) => {
                        info!(saved_id = %row.id, new_version = row.version, "update_ok");
                        Ok(row.try_into()?)
                    }
                    Err(diesel::result::Error::NotFound) => {
                        // Check if it exists but version mismatch
                        let exists = diesel::select(diesel::dsl::exists(
                            stages.filter(id.eq(inner.get_id())),
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
            // Case 2: INSERT with specific ID (e.g. Migration/Cloning)
            IdVersion::NewWithId(new_id) => {
                let row = diesel::insert_into(stages)
                    .values((id.eq(new_id), w))
                    .returning((
                        id,
                        version,
                        tournament_id,
                        number,
                        num_groups,
                        created_at,
                        updated_at,
                    ))
                    .get_result::<DbStage>(&mut conn)
                    .await
                    .map_err(map_db_err)?;

                info!(saved_id = %row.id, "insert_ok");
                Ok(row.try_into()?)
            }
            // Case 3: INSERT with DB-generated ID (Standard)
            IdVersion::New => {
                let row = diesel::insert_into(stages)
                    .values(w)
                    .returning((
                        id,
                        version,
                        tournament_id,
                        number,
                        num_groups,
                        created_at,
                        updated_at,
                    ))
                    .get_result::<DbStage>(&mut conn)
                    .await
                    .map_err(map_db_err)?;

                info!(saved_id = %row.id, "insert_ok");
                Ok(row.try_into()?)
            }
        }
    }

    #[instrument(name = "db.stage.list", skip(self, t_id))]
    async fn list_stages_of_tournament(&self, t_id: Uuid) -> DbResult<Vec<Stage>> {
        let mut conn = self.new_connection().await?;

        let rows = stages
            .filter(tournament_id.eq(t_id))
            .order(number.asc())
            .load::<DbStage>(&mut conn)
            .await
            .map_err(map_db_err)?;

        info!(count = rows.len(), "list_ok");
        rows.into_iter().map(Stage::try_from).collect()
    }
}
