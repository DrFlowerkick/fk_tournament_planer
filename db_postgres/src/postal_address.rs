// implementation of postal address port

use crate::{
    PgDb, escape_like, map_db_err,
    schema::{postal_addresses, postal_addresses::dsl::*},
};
use app_core::{DbError, DbResult, DbpPostalAddress, PostalAddress, utils::id_version::IdVersion};
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

// ------------------- DB-Row (SELECT/RETURNING) -------------------
#[derive(Debug, Queryable)]
pub struct DbPostalAddress {
    pub id: Uuid,
    pub version: i64,
    pub name: Option<String>,
    pub street: String,
    pub postal_code: String,
    pub locality: String,
    pub region: Option<String>,
    pub country: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Mapping DB -> Core
impl TryFrom<DbPostalAddress> for PostalAddress {
    type Error = DbError;

    fn try_from(r: DbPostalAddress) -> Result<Self, Self::Error> {
        if r.id.is_nil() {
            return Err(DbError::NilRowId);
        }
        if r.version < 0 {
            return Err(DbError::NegativeRowVersion);
        }
        if r.version > u32::MAX as i64 {
            return Err(DbError::RowVersionOutOfRange);
        }
        let id_version = IdVersion::new(r.id, r.version as u32);
        let mut pa = PostalAddress::new(id_version);
        pa.set_name(r.name.unwrap_or_default())
            .set_street(r.street)
            .set_postal_code(r.postal_code)
            .set_locality(r.locality)
            .set_region(r.region.unwrap_or_default())
            .set_country(r.country);
        pa.validate()?;
        Ok(pa)
    }
}

// ------------------- INSERT / UPDATE -------------------
// treat_none_as_null: None -> NULL (für optionale Felder)
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = postal_addresses)]
#[diesel(treat_none_as_null = true)]
pub struct WriteDbPostalAddress<'a> {
    // do NIT set version -> DEFAULT 0 from migration
    pub name: Option<&'a str>,
    pub street: &'a str,
    pub postal_code: &'a str,
    pub locality: &'a str,
    pub region: Option<&'a str>,
    pub country: &'a str,
}

// Mapping Core -> DB
impl<'a> From<&'a PostalAddress> for WriteDbPostalAddress<'a> {
    fn from(p: &'a PostalAddress) -> Self {
        WriteDbPostalAddress {
            name: p.get_name(),
            street: p.get_street(),
            postal_code: p.get_postal_code(),
            locality: p.get_locality(),
            region: p.get_region(),
            country: p.get_country(),
        }
    }
}

// ------------------- Impl trait --------------------

#[async_trait]
impl DbpPostalAddress for PgDb {
    #[instrument(name = "db.pa.get", skip(self), fields(id = %pa_id))]
    async fn get_postal_address(&self, pa_id: Uuid) -> DbResult<Option<PostalAddress>> {
        let mut conn = self.new_connection().await?;
        let res = postal_addresses
            .filter(id.eq(pa_id))
            .first::<DbPostalAddress>(&mut conn)
            .await
            .optional()
            .map_err(map_db_err)?;

        match res {
            Some(res) => {
                let res = PostalAddress::try_from(res)?;
                debug!("row_found");
                Ok(Some(res))
            }
            None => {
                debug!("row_not_found");
                Ok(None)
            }
        }
    }

    #[instrument(
        name = "db.pa.save",
        skip(self, address),
        fields(
            id = ?address.get_id(),
            version = address.get_version(),
            is_new = address.get_id().is_none()
        )
    )]
    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress> {
        let mut conn = self.new_connection().await?;
        let w = WriteDbPostalAddress::from(address);

        if let IdVersion::Existing(inner) = address.get_id_version() {
            // UPDATE with optimistic locking
            let res = diesel::update(
                postal_addresses.filter(
                    id.eq(inner.get_id())
                        .and(version.eq(inner.get_version() as i64)),
                ),
            )
            .set((w, version.eq(sql::<BigInt>("version + 1"))))
            .returning((
                id,
                version,
                name,
                street,
                postal_code,
                locality,
                region,
                country,
                created_at,
                updated_at,
            ))
            .get_result::<DbPostalAddress>(&mut conn)
            .await;

            match res {
                Ok(row) => {
                    info!(saved_id = %row.id, new_version = row.version, "update_ok");
                    Ok(row.try_into()?)
                }
                Err(diesel::result::Error::NotFound) => {
                    // Distinguish lock conflict from missing row
                    let exists = diesel::select(diesel::dsl::exists(
                        postal_addresses.filter(id.eq(inner.get_id())),
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
            let row = diesel::insert_into(postal_addresses)
                .values(w)
                .returning((
                    id,
                    version,
                    name,
                    street,
                    postal_code,
                    locality,
                    region,
                    country,
                    created_at,
                    updated_at,
                ))
                .get_result::<DbPostalAddress>(&mut conn)
                .await
                .map_err(map_db_err)?;
            info!(saved_id = %row.id, "insert_ok");
            Ok(row.try_into()?)
        }
    }

    #[instrument(
        name = "db.pa.list",
        skip(self, name_filter, limit),
        fields(
            q_len = name_filter.map(|s| s.len()).unwrap_or(0),
            limit = limit.unwrap_or(10)
        )
    )]
    async fn list_postal_addresses(
        &self,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<PostalAddress>> {
        let mut conn = self.new_connection().await?;

        let mut query = postal_addresses.into_boxed::<diesel::pg::Pg>();

        if let Some(f) = name_filter {
            if !f.is_empty() {
                // Case-insensitive "contains" match; escape special chars for LIKE
                let pattern = format!("%{}%", escape_like(f));
                debug!("apply_name_filter");
                query = query.filter(name.like(pattern));
            }
        }

        if let Some(lim) = limit {
            query = query.limit(lim as i64);
        }

        let rows = query
            .order((name.asc().nulls_last(), created_at.asc()))
            .load::<DbPostalAddress>(&mut conn)
            .await
            .map_err(map_db_err)?;

        info!(count = rows.len(), "list_ok");
        Ok(rows
            .into_iter()
            .map(PostalAddress::try_from)
            .collect::<Result<Vec<PostalAddress>, _>>()?)
    }
}
