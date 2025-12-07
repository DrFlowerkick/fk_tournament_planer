//! Parameters module for shared query parameter definitions and utilities.

use leptos::Params;
use leptos_router::params::Params;
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct AddressParams {
    pub address_id: Option<Uuid>,
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportParams {
    pub sport_id: Option<Uuid>,
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportConfigParams {
    pub sport_config_id: Option<Uuid>,
}
