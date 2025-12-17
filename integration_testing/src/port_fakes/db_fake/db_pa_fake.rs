//! Fakes for DbpPostalAddress port

use super::FakeDatabasePort;
use app_core::{DbError, DbResult, DbpPostalAddress, PostalAddress, utils::id_version::IdVersion};
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
        if let Some(id) = address.get_id() {
            if let Some(existing) = guard.get(&id) {
                let version = existing.get_version().unwrap() + 1;
                new.set_id_version(IdVersion::new(id, version));
            } else {
                // This case can happen if an ID is provided but not found (e.g., update on non-existent row)
                // For simplicity, we treat it as an insert, but a real DB might error.
                new.set_id_version(IdVersion::new(id, 0));
            }
        } else {
            new.set_id_version(IdVersion::new(Uuid::new_v4(), 0));
        }

        guard.insert(new.get_id().unwrap(), new.clone());
        Ok(new)
    }

    async fn list_postal_addresses(
        &self,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<PostalAddress>> {
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
        Ok(rows)
    }
}
