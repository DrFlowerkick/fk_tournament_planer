//! Fakes for DbpPostalAddress port

use super::FakeDatabasePort;
use app_core::{
    DbError, DbResult, DbpPostalAddress, PostalAddress,
    utils::{id_version::IdVersion, traits::ObjectIdVersion},
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
impl DbpPostalAddress for FakeDatabasePort {
    async fn get_postal_address(&self, id: Uuid) -> DbResult<Option<PostalAddress>> {
        let mut guard = self.fail_next_get_pa.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected get failure".into()));
        }
        Ok(self.postal_addresses.lock().unwrap().get(&id).cloned())
    }

    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress> {
        let mut guard = self.fail_next_save_pa.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected save failure".into()));
        }

        let mut guard = self.postal_addresses.lock().unwrap();
        let mut new = address.clone();

        match address.get_id_version() {
            IdVersion::Existing(inner) => {
                if let Some(existing) = guard.get(&inner.get_id()) {
                    let existing_v = existing.get_version().unwrap_or(0);
                    let update_v = inner.get_version();

                    if existing_v != update_v {
                        return Err(DbError::OptimisticLockConflict);
                    }

                    new.set_id_version(IdVersion::new(inner.get_id(), Some(existing_v + 1)));
                } else {
                    return Err(DbError::NotFound);
                }
            }
            IdVersion::NewWithId(id) => {
                if guard.contains_key(&id) {
                    return Err(DbError::Other(format!(
                        "PostalAddress with ID {} already exists",
                        id
                    )));
                }
                new.set_id_version(IdVersion::new(id, Some(0)));
            }
        }

        guard.insert(new.get_id(), new.clone());
        Ok(new)
    }

    async fn list_postal_address_ids(
        &self,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<Uuid>> {
        let mut guard = self.fail_next_list_pa.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other("injected list failure".into()));
        }

        let filter = name_filter.map(|s| s.to_lowercase());
        let mut rows: Vec<_> = self
            .postal_addresses
            .lock()
            .unwrap()
            .values()
            .filter(|a| {
                if let Some(ref f) = filter {
                    a.get_name().to_lowercase().contains(f)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        rows.sort_by(|a, b| match a.get_name().cmp(b.get_name()) {
            std::cmp::Ordering::Equal => a.get_id().cmp(&b.get_id()),
            cmp => cmp,
        });

        if let Some(l) = limit {
            rows.truncate(l);
        }
        Ok(rows.into_iter().map(|a| a.get_id()).collect())
    }
}
