// contains core functionality

mod entrant;
mod group;
mod match_;
mod ports;
mod postal_address;
mod round;
mod scoring;
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
pub use stage::*;
pub use timing::*;
pub use tournament::*;

use std::sync::Arc;

/// Core does provide on server context:
/// - API to create, modify, delete, schedule, and orchestrate tournaments
/// - API to create, modify, delete entrants and members
/// - API for entrants to participate at a tournament
/// - input validators
/// - Administration API
/// Core holds connections to all required ports (e.g. data base, sending email,
/// connectors to sport specific ranking systems).
/// Core does provide on client context:
/// - input validators
pub struct Core<S> {
    state: S,
    pub database: Arc<dyn DatabasePort>,
    pub client_registry: Arc<dyn ClientRegistryPort>,
}

impl<S> Core<S> {
    fn switch_state<N>(&self, new_state: N) -> Core<N> {
        Core {
            state: new_state,
            database: self.database.clone(),
            client_registry: self.client_registry.clone(),
        }
    }
}

// ToDo: we probably need some kind of configuration to provide init values for port creation. Or we do everything via .env.
pub struct InitState {}
pub type CoreState = Arc<Core<InitState>>;

pub struct NoDB {}
pub struct NoCR {}

pub struct DynDB(Arc<dyn DatabasePort>);
pub struct DynCR(Arc<dyn ClientRegistryPort>);

pub struct CoreBuilder<DB, CR> {
    state_db: DB,
    state_cr: CR,
}

impl CoreBuilder<NoDB, NoCR> {
    pub fn new() -> Self {
        CoreBuilder {
            state_db: NoDB {},
            state_cr: NoCR {},
        }
    }
}

impl<DB, CR> CoreBuilder<DB, CR> {
    pub fn set_db(self, database: Arc<dyn DatabasePort>) -> CoreBuilder<DynDB, CR> {
        CoreBuilder {
            state_db: DynDB(database),
            state_cr: self.state_cr,
        }
    }

    pub fn set_cr(self, client_registry: Arc<dyn ClientRegistryPort>) -> CoreBuilder<DB, DynCR> {
        CoreBuilder {
            state_db: self.state_db,
            state_cr: DynCR(client_registry),
        }
    }
}

impl CoreBuilder<DynDB, DynCR> {
    pub fn build(self) -> Core<InitState> {
        Core {
            state: InitState {},
            database: self.state_db.0,
            client_registry: self.state_cr.0,
        }
    }
}
