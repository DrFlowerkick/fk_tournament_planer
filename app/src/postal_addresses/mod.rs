// web ui for adding and modifying postal addresses

mod edit;
mod search;
pub mod server_fn;

#[cfg(feature = "test-mock")]
pub mod server_fn_test_support;

pub use edit::*;
pub use search::*;

use leptos::Params;
use leptos_router::params::Params;
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct AddressParams {
    pub address_id: Option<Uuid>,
}
