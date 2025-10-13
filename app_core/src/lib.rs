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

#[derive(Clone)]
pub struct ServerContext {
    database: Arc<dyn DatabasePort>,
    client_registry: Arc<dyn ClientRegistryPort>,
}

#[derive(Clone)]
pub struct ClientContext {}

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
pub struct Core<S, C: Clone> {
    state: S,
    context: C,
}

impl<S, C: Clone> Core<S, C> {
    fn switch_state<N>(&self, new_state: N) -> Core<N, C> {
        Core {
            state: new_state,
            context: self.context.clone(),
        }
    }
}

impl<S> Core<S, ServerContext> {
    pub fn database(&self) -> Arc<dyn DatabasePort> {
        self.context.database.clone()
    }
    pub fn client_registry(&self) -> Arc<dyn ClientRegistryPort> {
        self.context.client_registry.clone()
    }
}

// ToDo: we probably need some kind of configuration to provide init values for port creation. Or we do everything via .env.
pub struct InitState {}
pub type CoreServerState = Arc<Core<InitState, ServerContext>>;
pub type CoreClientState = Arc<Core<InitState, ClientContext>>;

pub struct NoContext {}
pub struct BuildServerContext {}
pub struct NoDB {}
pub struct NoCR {}

pub struct DynDB {
    database: Arc<dyn DatabasePort>,
}
pub struct DynCR {
    client_registry: Arc<dyn ClientRegistryPort>,
}

pub struct CoreBuilder<DB, CR, CO> {
    state_db: DB,
    state_cr: CR,
    context: CO,
}

impl CoreBuilder<NoDB, NoCR, NoContext> {
    pub fn new() -> Self {
        CoreBuilder {
            state_db: NoDB {},
            state_cr: NoCR {},
            context: NoContext {},
        }
    }
    pub fn server_context(self) -> CoreBuilder<NoDB, NoCR, BuildServerContext> {
        CoreBuilder {
            state_db: self.state_db,
            state_cr: self.state_cr,
            context: BuildServerContext {},
        }
    }
    pub fn client_context(self) -> CoreBuilder<NoDB, NoCR, ClientContext> {
        CoreBuilder {
            state_db: self.state_db,
            state_cr: self.state_cr,
            context: ClientContext {},
        }
    }
}

impl CoreBuilder<NoDB, NoCR, ClientContext> {
    pub fn build(self) -> Core<InitState, ClientContext> {
        Core {
            state: InitState {},
            context: self.context,
        }
    }
}

impl<DB, CR> CoreBuilder<DB, CR, BuildServerContext> {
    pub fn set_db(
        self,
        database: Arc<dyn DatabasePort>,
    ) -> CoreBuilder<DynDB, CR, BuildServerContext> {
        CoreBuilder {
            state_db: DynDB { database },
            state_cr: self.state_cr,
            context: self.context,
        }
    }

    pub fn set_cr(
        self,
        client_registry: Arc<dyn ClientRegistryPort>,
    ) -> CoreBuilder<DB, DynCR, BuildServerContext> {
        CoreBuilder {
            state_db: self.state_db,
            state_cr: DynCR { client_registry },
            context: self.context,
        }
    }
}

impl CoreBuilder<DynDB, DynCR, BuildServerContext> {
    pub fn build(self) -> Core<InitState, ServerContext> {
        Core {
            state: InitState {},
            context: ServerContext {
                database: self.state_db.database,
                client_registry: self.state_cr.client_registry,
            },
        }
    }
}
