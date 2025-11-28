// contains core functionality

mod entrant;
mod group;
mod match_;
mod ports;
mod postal_address;
mod round;
mod scoring;
mod sport_config;
mod stage;
mod timing;
mod tournament;
pub mod utils;

pub use entrant::*;
pub use group::*;
pub use match_::*;
pub use ports::*;
pub use postal_address::*;
pub use round::*;
pub use scoring::*;
pub use sport_config::*;
pub use stage::*;
pub use timing::*;
pub use tournament::*;

use std::sync::Arc;

/// Core does provide on server context:
/// - API to create, edit, delete, schedule, and orchestrate tournaments
/// - API to create, edit, delete entrants and members
/// - API for entrants to participate at a tournament
/// - input validators
/// - Administration API
///
/// Core holds connections to all required ports (e.g. data base, sending email,
///   connectors to sport specific ranking systems).
///
/// Core does provide on client context:
/// - input validators
pub struct Core<S> {
    state: S,
    pub database: Arc<dyn DatabasePort>,
    pub client_registry: Arc<dyn ClientRegistryPort>,
    pub sport_plugins: Arc<dyn SportPluginManagerPort>,
}

impl<S> Core<S> {
    fn switch_state<N>(&self, new_state: N) -> Core<N> {
        Core {
            state: new_state,
            database: self.database.clone(),
            client_registry: self.client_registry.clone(),
            sport_plugins: self.sport_plugins.clone(),
        }
    }
}

// ToDo: we probably need some kind of configuration to provide init values for port creation. Or we do everything via .env.
pub struct InitState {}
pub type CoreState = Arc<Core<InitState>>;

pub struct NoDB {}
pub struct NoCR {}
pub struct NoSPM {}

pub struct DynDB(Arc<dyn DatabasePort>);
pub struct DynCR(Arc<dyn ClientRegistryPort>);
pub struct DynSPM(Arc<dyn SportPluginManagerPort>);

pub struct CoreBuilder<DB, CR, SPM> {
    state_db: DB,
    state_cr: CR,
    state_spm: SPM,
}

impl CoreBuilder<NoDB, NoCR, NoSPM> {
    pub fn new() -> Self {
        CoreBuilder {
            state_db: NoDB {},
            state_cr: NoCR {},
            state_spm: NoSPM {},
        }
    }
}

impl Default for CoreBuilder<NoDB, NoCR, NoSPM> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB, CR, SPM> CoreBuilder<DB, CR, SPM> {
    pub fn set_db(self, database: Arc<dyn DatabasePort>) -> CoreBuilder<DynDB, CR, SPM> {
        CoreBuilder {
            state_db: DynDB(database),
            state_cr: self.state_cr,
            state_spm: self.state_spm,
        }
    }

    pub fn set_cr(
        self,
        client_registry: Arc<dyn ClientRegistryPort>,
    ) -> CoreBuilder<DB, DynCR, SPM> {
        CoreBuilder {
            state_db: self.state_db,
            state_cr: DynCR(client_registry),
            state_spm: self.state_spm,
        }
    }

    pub fn set_spm(
        self,
        sport_plugin_manager: Arc<dyn SportPluginManagerPort>,
    ) -> CoreBuilder<DB, CR, DynSPM> {
        CoreBuilder {
            state_db: self.state_db,
            state_cr: self.state_cr,
            state_spm: DynSPM(sport_plugin_manager),
        }
    }
}

impl CoreBuilder<DynDB, DynCR, DynSPM> {
    pub fn build(self) -> Core<InitState> {
        Core {
            state: InitState {},
            database: self.state_db.0,
            client_registry: self.state_cr.0,
            sport_plugins: self.state_spm.0,
        }
    }
}
