// implementation of postal address port

use crate::{
    PgDb, escape_like, map_db_err,
    schema::{postal_addresses, postal_addresses::dsl::*},
};
use app_core::{DbError, DbResult, DbpPostalAddress, PostalAddress};
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
    pub street_address: String,
    pub postal_code: String,
    pub address_locality: String,
    pub address_region: Option<String>,
    pub address_country: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Mapping DB -> Core
impl From<DbPostalAddress> for PostalAddress {
    fn from(r: DbPostalAddress) -> Self {
        Self {
            id: r.id,
            version: r.version,
            name: r.name,
            street_address: r.street_address,
            postal_code: r.postal_code,
            address_locality: r.address_locality,
            address_region: r.address_region,
            address_country: r.address_country,
        }
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
    pub street_address: &'a str,
    pub postal_code: &'a str,
    pub address_locality: &'a str,
    pub address_region: Option<&'a str>,
    pub address_country: &'a str,
}

// Mapping Core -> DB
impl<'a> From<&'a PostalAddress> for WriteDbPostalAddress<'a> {
    fn from(p: &'a PostalAddress) -> Self {
        WriteDbPostalAddress {
            name: p.name.as_deref(),
            street_address: &p.street_address,
            postal_code: &p.postal_code,
            address_locality: &p.address_locality,
            address_region: p.address_region.as_deref(),
            address_country: &p.address_country,
        }
    }
}

// ------------------- Impl trait --------------------

const NEW_VERSION: i64 = -1;

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

        match &res {
            Some(_) => debug!("row_found"),
            None => debug!("row_not_found"),
        }
        Ok(res.map(PostalAddress::from))
    }

    #[instrument(
        name = "db.pa.save",
        skip(self, address),
        fields(
            id = %address.id,
            version = address.version,
            is_new = address.id.is_nil() || address.version == NEW_VERSION
        )
    )]
    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress> {
        let mut conn = self.new_connection().await?;
        let w = WriteDbPostalAddress::from(address);

        if address.id.is_nil() || address.version == NEW_VERSION {
            // INSERT
            let row = diesel::insert_into(postal_addresses)
                .values(w)
                .returning((
                    id,
                    version,
                    name,
                    street_address,
                    postal_code,
                    address_locality,
                    address_region,
                    address_country,
                    created_at,
                    updated_at,
                ))
                .get_result::<DbPostalAddress>(&mut conn)
                .await
                .map_err(map_db_err)?;
            info!(saved_id = %row.id, "insert_ok");
            Ok(row.into())
        } else {
            // UPDATE with optimistic locking
            let res = diesel::update(
                postal_addresses.filter(id.eq(address.id).and(version.eq(address.version))),
            )
            .set((w, version.eq(sql::<BigInt>("version + 1"))))
            .returning((
                id,
                version,
                name,
                street_address,
                postal_code,
                address_locality,
                address_region,
                address_country,
                created_at,
                updated_at,
            ))
            .get_result::<DbPostalAddress>(&mut conn)
            .await;

            match res {
                Ok(row) => {
                    info!(saved_id = %row.id, new_version = row.version, "update_ok");
                    Ok(row.into())
                }
                Err(diesel::result::Error::NotFound) => {
                    // Distinguish lock conflict from missing row
                    let exists = diesel::select(diesel::dsl::exists(
                        postal_addresses.filter(id.eq(address.id)),
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
        Ok(rows.into_iter().map(PostalAddress::from).collect())
    }
}
