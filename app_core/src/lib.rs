// contains core functionality

mod entrant;
mod group;
mod postal_address;
mod match_;
mod ports;
mod round;
mod scoring;
mod stage;
mod timing;
mod tournament;

pub use entrant::*;
pub use group::*;
pub use postal_address::*;
pub use match_::*;
pub use ports::*;
pub use round::*;
pub use scoring::*;
pub use stage::*;
pub use timing::*;
pub use tournament::*;

use anyhow::Result;
use std::sync::Arc;

/// Core does provide:
/// - API to create, modify, delete, schedule, and orchestrate tournaments
/// - API to create, modify, delete entrants and members
/// - API for entrants to participate at a tournament
/// - Administration API
/// Core holds connections to all required ports (e.g. data base, sending email,
/// connectors to sport specific ranking systems).
/// Core is a server side sync + send async object.
pub struct Core<S> {
    pub state: S,
    data_base: Arc<dyn DatabasePort>,
    client_registry: Arc<dyn ClientRegistryPort>,
}

impl<S> Core<S> {
    fn switch_state<N>(&self, new_state: N) -> Core<N> {
        Core {
            state: new_state,
            data_base: self.data_base.clone(),
            client_registry: self.client_registry.clone(),
        }
    }
}

// ToDo: perhaps we use Builder Pattern?
// ToDo: we probably need some kind of configuration to provide init values for port creation
pub struct InitState {}

impl Core<InitState> {
    pub async fn new(data_base: Arc<dyn DatabasePort>, client_registry: Arc<dyn ClientRegistryPort>) -> Result<Core<InitState>> {
        Ok(Core {
            state: InitState {},
            data_base,
            client_registry,
        })
    }
}
