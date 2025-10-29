use app_core::{ DatabasePort, DbError, DbResult,
    DbpPostalAddress, PostalAddress,  utils::id_version::IdVersion,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use async_trait::async_trait;

#[derive(Clone, Default)]
pub struct FakeDatabasePort {
    inner: Arc<Mutex<HashMap<Uuid, PostalAddress>>>,
    fail_next_get: Arc<Mutex<bool>>,
    fail_next_save: Arc<Mutex<bool>>,
    fail_next_list: Arc<Mutex<bool>>,
}

impl FakeDatabasePort {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DbpPostalAddress for FakeDatabasePort {
    async fn get_postal_address(&self, id: Uuid) -> DbResult<Option<PostalAddress>> {
        let mut guard = self.fail_next_get.lock().unwrap();
        if *guard {
            *guard = false;
            // Construct a deterministic DbError variant from your enum.
            return Err(DbError::Other(anyhow::anyhow!("injected get failure")));
        }
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }

    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress> {
        let mut guard = self.fail_next_save.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other(anyhow::anyhow!("injected save failure")));
        }

        let mut guard = self.inner.lock().unwrap();
        let mut new = address.clone();
        if let Some(id) = address.get_id()
            && let Some(existing) = guard.get(&id)
        {
            let version = existing.get_version().unwrap() + 1;
            new.set_id_version(IdVersion::new(id, version));
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
        let mut guard = self.fail_next_list.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other(anyhow::anyhow!("injected list failure")));
        }

        let filter = name_filter.map(|s| s.to_lowercase());
        let mut rows: Vec<_> = self
            .inner
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

        // deterministic order: by name, then id
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

// Blanket impl: your DatabasePort is a supertrait of DbpPostalAddress.
#[async_trait]
impl DatabasePort for FakeDatabasePort {
    async fn ping_db(&self) -> DbResult<()> {
        Ok(())
    }
}