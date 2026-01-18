mod db_pa_fake;
mod db_sc_fake;
mod db_stage_fake;
mod db_tb_fake;

use crate::port_fakes::MockSport;
use app_core::{
    ClientRegistryPort, Core, CoreBuilder, CrError, CrMsg, CrResult, CrTopic, DatabasePort,
    DbResult, InitState, PostalAddress, PostalAddressState, SportConfig, SportConfigState,
    SportPluginManagerPort, Stage, StageState, TournamentBase, TournamentBaseState, TournamentMode,
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
    // for tournament bases
    tournament_bases: Arc<Mutex<HashMap<Uuid, TournamentBase>>>,
    fail_next_get_tb: Arc<Mutex<bool>>,
    fail_next_save_tb: Arc<Mutex<bool>>,
    fail_next_list_tb: Arc<Mutex<bool>>,
    // for stage
    stages: Arc<Mutex<HashMap<Uuid, Stage>>>,
    fail_next_get_stage: Arc<Mutex<bool>>,
    fail_next_save_stage: Arc<Mutex<bool>>,
    fail_next_list_stage: Arc<Mutex<bool>>,
}

impl FakeDatabasePort {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Postal Address Helpers ---

    pub fn seed_postal_address(&self, mut addr: PostalAddress) -> Uuid {
        assert!(addr.get_id().is_none());
        let id = Uuid::new_v4();
        let id_version = IdVersion::new(id, Some(0));
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
        assert!(config.get_id().is_none());
        let id = Uuid::new_v4();
        let id_version = IdVersion::new(id, Some(0));
        config.set_id_version(id_version);
        self.sport_configs.lock().unwrap().insert(id, config);
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

    // --- Tournament Base Helpers ---
    pub fn seed_tournament_base(&self, mut tb: TournamentBase) -> Uuid {
        assert!(tb.get_id().is_none());
        let id = Uuid::new_v4();
        let id_version = IdVersion::new(id, Some(0));
        tb.set_id_version(id_version);
        self.tournament_bases.lock().unwrap().insert(id, tb);
        id
    }

    pub fn fail_get_tb_once(&self) {
        *self.fail_next_get_tb.lock().unwrap() = true;
    }
    pub fn fail_save_tb_once(&self) {
        *self.fail_next_save_tb.lock().unwrap() = true;
    }
    pub fn fail_list_tb_once(&self) {
        *self.fail_next_list_tb.lock().unwrap() = true;
    }

    // --- Stage Helpers ---
    pub fn seed_stage(&self, mut stage: Stage) -> Uuid {
        assert!(stage.get_id().is_none());
        let id = Uuid::new_v4();
        let id_version = IdVersion::new(id, Some(0));
        stage.set_id_version(id_version);
        self.stages.lock().unwrap().insert(id, stage);
        id
    }

    pub fn fail_get_stage_once(&self) {
        *self.fail_next_get_stage.lock().unwrap() = true;
    }
    pub fn fail_save_stage_once(&self) {
        *self.fail_next_save_stage.lock().unwrap() = true;
    }
    pub fn fail_list_stage_once(&self) {
        *self.fail_next_list_stage.lock().unwrap() = true;
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
    async fn publish(&self, _topic: CrTopic, notice: CrMsg) -> CrResult<()> {
        let mut guard = self.fail_next_publish.lock().unwrap();
        if *guard {
            *guard = false;
            return Err(CrError::from(anyhow::anyhow!("injected publish failure")));
        }
        self.published.lock().unwrap().push(notice);
        Ok(())
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

/// Helper: build a Core<InitState> wired with our trait fakes.
pub fn make_core_with_fakes() -> (
    Core<InitState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
    Arc<SportPluginManagerMap>,
) {
    let db = Arc::new(FakeDatabasePort::new());
    let cr = Arc::new(FakeClientRegistryPort::new());
    let mut spm = SportPluginManagerMap::new();
    spm.register(Arc::new(MockSport {
        id: Uuid::new_v4(),
        name: "Mock Sport",
    }))
    .unwrap();
    let spm = Arc::new(spm);
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
    Core<SportConfigState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
) {
    let (core, db, cr, _spm) = make_core_with_fakes();
    (core.as_sport_config_state(), db, cr)
}

pub fn make_sport_config(name: &str, sport_core: &Core<SportConfigState>) -> SportConfig {
    let mut sc = SportConfig::new(IdVersion::New);
    let sport_id = sport_core.sport_plugins.list()[0]
        .get_id_version()
        .get_id()
        .unwrap();
    sc.set_name(name).set_sport_id(sport_id);
    sc
}

pub fn make_core_tournament_base_state_with_fakes() -> (
    Core<TournamentBaseState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
) {
    let (core, db, cr, _spm) = make_core_with_fakes();
    (core.as_tournament_base_state(), db, cr)
}

pub fn make_tournament_base(name: &str, sport_core: &Core<TournamentBaseState>) -> TournamentBase {
    let mut tb = TournamentBase::new(IdVersion::New);
    let sport_id = sport_core.sport_plugins.list()[0]
        .get_id_version()
        .get_id()
        .unwrap();
    tb.set_name(name)
        .set_sport_id(sport_id)
        .set_num_entrants(10);
    tb
}

pub async fn make_core_stage_state_with_fakes() -> (
    Core<StageState>,
    Arc<FakeDatabasePort>,
    Arc<FakeClientRegistryPort>,
) {
    let (core, db, cr, spm) = make_core_with_fakes();

    let sport_id = spm.list()[0].get_id_version().get_id().unwrap();

    let mut tb = TournamentBase::new(IdVersion::New);
    tb.set_name("Stage Context Tournament")
        .set_sport_id(sport_id)
        // IMPORTANT: Must be at least 2x max(num_groups) used in tests.
        // Tests use up to 8 groups, so 16 would be min. Using 32 to be safe.
        .set_num_entrants(32)
        // IMPORTANT: Use a mode that allows multiple stages and multiple groups.
        // TwoPoolStagesAndFinalStage allows 3 stages (indices 0, 1, 2).
        .set_tournament_mode(TournamentMode::TwoPoolStagesAndFinalStage);

    let t_id = db.seed_tournament_base(tb);

    let mut core_stage_state = core
        .as_stage_state(t_id)
        .await
        .expect("Test setup failed: could not switch to stage state");

    core_stage_state.get_mut().set_tournament_id(t_id);

    (core_stage_state, db, cr)
}
