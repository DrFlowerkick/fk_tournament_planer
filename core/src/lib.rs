// contains core functionality

mod entrant;
mod group;
mod location;
mod match_;
mod round;
mod scoring;
mod stage;
mod timing;
mod tournament;

pub use entrant::*;
pub use group::*;
pub use location::*;
pub use match_::*;
pub use round::*;
pub use scoring::*;
pub use stage::*;
pub use timing::*;
pub use tournament::*;

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
}

impl<S> Core<S> {
    fn switch_state<N>(&self, new_state: N) -> Core<N> {
        Core { state: new_state }
    }
}
