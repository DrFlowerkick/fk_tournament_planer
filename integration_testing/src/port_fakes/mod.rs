mod db_pa_fake;
mod db_sc_fake;

use app_core::{
    ClientRegistryPort, Core, CoreBuilder, CrMsg, CrTopic, DatabasePort, DbResult, InitState,
    PostalAddress, PostalAddressState, SportConfig, utils::id_version::IdVersion,
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

pub fn make_core_sport_config_state_with_fakes() -> (
    Core<app_core::SportConfigState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
) {
    let (core, db, cr, _spm) = make_core_with_fakes();
    (core.as_sport_config_state(), db, cr)
}

pub fn make_sport_config(name: &str, sport_id: Uuid) -> SportConfig {
    SportConfig {
        id_version: IdVersion::New,
        sport_id,
        name: name.to_string(),
        config: serde_json::Value::Null,
    }
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
