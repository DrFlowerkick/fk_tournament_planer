use app_core::{
    ClientRegistryPort, Core, CoreBuilder, CrMsg, CrTopic, DatabasePort, DbError, DbResult,
    DbpPostalAddress, DbpSportConfig, InitState, PostalAddress, PostalAddressState, SportConfig,
    utils::id_version::IdVersion,
};
use async_trait::async_trait;
use sport_plugin_manager::SportPluginManagerMap;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// In-memory DB fake implementing DbpPostalAddress and DbpSportConfig traits.
#[derive(Clone, Default)]
pub struct FakeDatabasePort {
    // for postal addresses
    postal_addresses: Arc<Mutex<HashMap<Uuid, PostalAddress>>>,
    fail_next_get_pa: Arc<Mutex<bool>>,
    fail_next_save_pa: Arc<Mutex<bool>>,
    fail_next_list_pa: Arc<Mutex<bool>>,
    // for sport configs
    sport_configs: Arc<Mutex<HashMap<Uuid, SportConfig>>>,
    fail_next_get_sc: Arc<Mutex<bool>>,
    fail_next_save_sc: Arc<Mutex<bool>>,
    fail_next_list_sc: Arc<Mutex<bool>>,
}

impl FakeDatabasePort {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Postal Address Helpers ---

    pub fn seed_postal_address(&self, mut addr: PostalAddress) -> Uuid {
        assert!(addr.get_id().is_none());
        let id = Uuid::new_v4();
        let id_version = IdVersion::new(id, 0);
        addr.set_id_version(id_version);
        self.postal_addresses
            .lock()
            .unwrap()
            .insert(addr.get_id().unwrap(), addr);
        id
    }

    pub fn fail_get_pa_once(&self) {
        *self.fail_next_get_pa.lock().unwrap() = true;
    }
    pub fn fail_save_pa_once(&self) {
        *self.fail_next_save_pa.lock().unwrap() = true;
    }
    pub fn fail_list_pa_once(&self) {
        *self.fail_next_list_pa.lock().unwrap() = true;
    }

    // --- Sport Config Helpers ---

    pub fn seed_sport_config(&self, mut config: SportConfig) -> Uuid {
        assert!(config.id_version.get_id().is_none());
        let id = Uuid::new_v4();
        let id_version = IdVersion::new(id, 0);
        config.id_version = id_version;
        self.sport_configs
            .lock()
            .unwrap()
            .insert(config.id_version.get_id().unwrap(), config);
        id
    }

    pub fn fail_get_sc_once(&self) {
        *self.fail_next_get_sc.lock().unwrap() = true;
    }
    pub fn fail_save_sc_once(&self) {
        *self.fail_next_save_sc.lock().unwrap() = true;
    }
    pub fn fail_list_sc_once(&self) {
        *self.fail_next_list_sc.lock().unwrap() = true;
    }
}

#[async_trait]
impl DbpPostalAddress for FakeDatabasePort {
    async fn get_postal_address(&self, id: Uuid) -> DbResult<Option<PostalAddress>> {
        let mut guard = self.fail_next_get_pa.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other(anyhow::anyhow!("injected get failure")));
        }
        Ok(self.postal_addresses.lock().unwrap().get(&id).cloned())
    }

    async fn save_postal_address(&self, address: &PostalAddress) -> DbResult<PostalAddress> {
        let mut guard = self.fail_next_save_pa.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other(anyhow::anyhow!("injected save failure")));
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
            return Err(DbError::Other(anyhow::anyhow!("injected list failure")));
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

#[async_trait]
impl DbpSportConfig for FakeDatabasePort {
    async fn get_sport_config(&self, id: Uuid) -> DbResult<Option<SportConfig>> {
        let mut guard = self.fail_next_get_sc.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other(anyhow::anyhow!("injected get failure")));
        }
        Ok(self.sport_configs.lock().unwrap().get(&id).cloned())
    }

    async fn save_sport_config(&self, config: &SportConfig) -> DbResult<SportConfig> {
        let mut guard = self.fail_next_save_sc.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other(anyhow::anyhow!("injected save failure")));
        }

        let mut guard = self.sport_configs.lock().unwrap();
        let mut new = config.clone();
        if let Some(id) = config.id_version.get_id() {
            if let Some(existing) = guard.get(&id) {
                let version = existing.id_version.get_version().unwrap() + 1;
                new.id_version = IdVersion::new(id, version);
            } else {
                new.id_version = IdVersion::new(id, 0);
            }
        } else {
            new.id_version = IdVersion::new(Uuid::new_v4(), 0);
        }

        guard.insert(new.id_version.get_id().unwrap(), new.clone());
        Ok(new)
    }

    async fn list_sport_configs(
        &self,
        name_filter: Option<&str>,
        limit: Option<usize>,
    ) -> DbResult<Vec<SportConfig>> {
        let mut guard = self.fail_next_list_sc.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(DbError::Other(anyhow::anyhow!("injected list failure")));
        }

        let filter = name_filter.map(|s| s.to_lowercase());
        let mut rows: Vec<_> = self
            .sport_configs
            .lock()
            .unwrap()
            .values()
            .filter(|sc| {
                if let Some(ref f) = filter {
                    sc.name.to_lowercase().contains(f)
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        rows.sort_by(|a, b| match a.name.cmp(&b.name) {
            std::cmp::Ordering::Equal => a.id_version.get_id().cmp(&b.id_version.get_id()),
            cmp => cmp,
        });

        if let Some(l) = limit {
            rows.truncate(l);
        }
        Ok(rows)
    }
}

// Blanket impl: your DatabasePort is a supertrait of DbpPostalAddress and DbpSportConfig.
#[async_trait]
impl DatabasePort for FakeDatabasePort {
    async fn ping_db(&self) -> DbResult<()> {
        Ok(())
    }
}

/// Minimal ClientRegistry fake.
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

/// Helper: build a Core<InitState> wired with our trait fakes.
pub fn make_core_with_fakes() -> (
    Core<InitState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
    Arc<SportPluginManagerMap>,
) {
    let db = Arc::new(FakeDatabasePort::new());
    let cr = Arc::new(FakeClientRegistryPort::new());
    let spm = Arc::new(SportPluginManagerMap::new());
    let core = CoreBuilder::new()
        .set_db(db.clone())
        .set_cr(cr.clone())
        .set_spm(spm.clone())
        .build();
    (core, db, cr, spm)
}

pub fn make_core_postal_address_state_with_fakes() -> (
    Core<PostalAddressState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
) {
    let (core, db, cr, _spm) = make_core_with_fakes();
    (core.as_postal_address_state(), db, cr)
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
