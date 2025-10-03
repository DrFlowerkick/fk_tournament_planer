// implementation of postal address port

use crate::{
    PgDb, map_db_err,
    schema::{postal_addresses, postal_addresses::dsl::*},
};
use app_core::{DbError, DbResult, DbpPostalAddress, PostalAddress};
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
    async fn get_postal_address(&self, pa_id: Uuid) -> DbResult<Option<PostalAddress>> {
        let mut conn = self.new_connection().await?;
        Ok(postal_addresses
            .filter(id.eq(pa_id))
            .first::<DbPostalAddress>(&mut conn)
            .await
            .optional()
            .map_err(map_db_err)?
            .map(PostalAddress::from))
    }
    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress> {
        let mut conn = self.new_connection().await?;
        let w = WriteDbPostalAddress::from(address);

        if address.id.is_nil() || address.version == NEW_VERSION {
            // INSERT
            Ok(diesel::insert_into(postal_addresses)
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
                .map_err(map_db_err)?
                .into())
        } else {
            // UPDATE if current version in table is equal to version in address
            let res = diesel::update(
                postal_addresses.filter(id.eq(address.id).and(version.eq(address.version))),
            )
            // increment version separately
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
                Ok(row) => Ok(row.into()),
                Err(diesel::result::Error::NotFound) => {
                    // check if ID exists
                    let exists = diesel::select(diesel::dsl::exists(
                        postal_addresses.filter(id.eq(address.id)),
                    ))
                    .get_result::<bool>(&mut conn)
                    .await
                    .map_err(map_db_err)?;

                    if exists {
                        Err(DbError::OptimisticLockConflict)
                    } else {
                        Err(DbError::NotFound)
                    }
                }
                Err(e) => Err(map_db_err(e)),
            }
        }
    }
}
