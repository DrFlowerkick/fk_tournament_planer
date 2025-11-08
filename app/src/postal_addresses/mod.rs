// web ui for adding and modifying postal addresses

mod edit;
mod search;
pub mod server_fn;

pub use edit::*;
pub use search::*;

use leptos::Params;
use leptos_router::params::Params;
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct AddressParams {
    pub uuid: Option<Uuid>,
}
