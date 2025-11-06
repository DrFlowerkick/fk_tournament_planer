use app_core::{
    ClientRegistryPort, Core, CoreBuilder, CrMsg, CrTopic, DatabasePort, DbError, DbResult,
    DbpPostalAddress, PostalAddress, PostalAddressState, utils::id_version::IdVersion,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use async_trait::async_trait;

/// In-memory DB fake implementing your DbpPostalAddress trait.
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
    // keep the seed fn, perhaps we kan use it later
    // ToDo: check, if seed should be removed
    #[allow(dead_code)]
    pub fn seed(&self, mut addr: PostalAddress) {
        assert!(addr.get_id().is_none());
        let id_version = IdVersion::new(Uuid::new_v4(), 0);
        addr.set_id_version(id_version);
        self.inner
            .lock()
            .unwrap()
            .insert(addr.get_id().unwrap(), addr);
    }

    pub fn fail_get_once(&self) {
        *self.fail_next_get.lock().unwrap() = true;
    }
    pub fn fail_save_once(&self) {
        *self.fail_next_save.lock().unwrap() = true;
    }
    pub fn fail_list_once(&self) {
        *self.fail_next_list.lock().unwrap() = true;
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

/// Minimal ClientRegistry fake (not used by these DB tests, but needed to build Core).
#[derive(Clone, Default)]
pub struct FakeClientRegistryPort {
    published: Arc<Mutex<Vec<CrMsg>>>,
    fail_next_publish: Arc<Mutex<bool>>,
}

impl FakeClientRegistryPort {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn published(&self) -> Vec<CrMsg> {
        self.published.lock().unwrap().clone()
    }
    pub fn clear(&self) {
        self.published.lock().unwrap().clear();
    }
    pub fn fail_publish_once(&self) {
        *self.fail_next_publish.lock().unwrap() = true;
    }
}

#[async_trait]
impl ClientRegistryPort for FakeClientRegistryPort {
    async fn publish(&self, _topic: CrTopic, notice: CrMsg) -> anyhow::Result<()> {
        let mut guard = self.fail_next_publish.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(anyhow::anyhow!("injected publish failure"));
        }
        self.published.lock().unwrap().push(notice);
        Ok(())
    }
}

/// Helper: build a Core<PostalAddressState> wired with our trait fakes.
pub fn make_core_with_fakes() -> (
    Core<PostalAddressState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
) {
    let db = Arc::new(FakeDatabasePort::new());
    let cr = Arc::new(FakeClientRegistryPort::new());

    let core = CoreBuilder::new()
        .set_db(db.clone())
        .set_cr(cr.clone())
        .build()
        .as_postal_address_state();
    (core, db, cr)
}

/// Convenience: construct a realistic PostalAddress for seeding.
pub fn make_addr(
    name: &str,
    street: &str,
    postal: &str,
    city: &str,
    region: &str,
    country: &str,
) -> PostalAddress {
    let mut pa = PostalAddress::new(IdVersion::New);
    pa.set_name(name)
        .set_street(street)
        .set_postal_code(postal)
        .set_locality(city)
        .set_region(region)
        .set_country(country);
    pa
}
