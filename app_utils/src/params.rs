//! Parameters module for shared query parameter definitions and utilities.

use crate::enum_utils::FilterLimit;
use app_core::TournamentState;
use leptos::prelude::*;
use leptos_router::{
    hooks::{use_params, use_query},
    params::Params,
};
use uuid::Uuid;

pub trait ParamQuery<T: Send + Sync + 'static>: Params + PartialEq + Send + Sync + 'static {
    fn key() -> &'static str;
    fn use_param_query() -> Signal<Option<T>>;
}

pub trait ParamQueryId: ParamQuery<Uuid> {
    fn get_id(&self) -> Option<Uuid>;
}

// ---------------------- Search Filters ----------------------

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct FilterNameQuery {
    pub filter_name: Option<String>,
}

impl ParamQuery<String> for FilterNameQuery {
    fn key() -> &'static str {
        "filter_name"
    }
    fn use_param_query() -> Signal<Option<String>> {
        let query = use_query::<Self>();
        Signal::derive(move || query.get().ok().and_then(|sn| sn.filter_name))
    }
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct FilterLimitQuery {
    pub filter_limit: Option<FilterLimit>,
}

impl ParamQuery<FilterLimit> for FilterLimitQuery {
    fn key() -> &'static str {
        "filter_limit"
    }
    fn use_param_query() -> Signal<Option<FilterLimit>> {
        let query = use_query::<Self>();
        Signal::derive(move || query.get().ok().and_then(|fl| fl.filter_limit))
    }
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct TournamentStateQuery {
    pub tournament_state: Option<TournamentState>,
}

impl ParamQuery<TournamentState> for TournamentStateQuery {
    fn key() -> &'static str {
        "tournament_state"
    }
    fn use_param_query() -> Signal<Option<TournamentState>> {
        let query = use_query::<Self>();
        Signal::derive(move || query.get().ok().and_then(|ts| ts.tournament_state))
    }
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct IncludeAdhocQuery {
    pub include_adhoc: Option<bool>,
}

impl ParamQuery<bool> for IncludeAdhocQuery {
    fn key() -> &'static str {
        "include_adhoc"
    }
    fn use_param_query() -> Signal<Option<bool>> {
        let query = use_query::<Self>();
        Signal::derive(move || query.get().ok().and_then(|ia| ia.include_adhoc))
    }
}

// ---------------------- Postal Address ----------------------

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct AddressIdQuery {
    pub address_id: Option<Uuid>,
}

impl ParamQuery<Uuid> for AddressIdQuery {
    fn key() -> &'static str {
        "address_id"
    }
    fn use_param_query() -> Signal<Option<Uuid>> {
        let query = use_query::<Self>();
        Signal::derive(move || query.with(|p| p.as_ref().ok().and_then(|params| params.address_id)))
    }
}

impl ParamQueryId for AddressIdQuery {
    fn get_id(&self) -> Option<Uuid> {
        self.address_id
    }
}

// ---------------------- Sport ----------------------

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportIdQuery {
    pub sport_id: Option<Uuid>,
}

impl ParamQuery<Uuid> for SportIdQuery {
    fn key() -> &'static str {
        "sport_id"
    }
    fn use_param_query() -> Signal<Option<Uuid>> {
        let query = use_query::<Self>();
        Signal::derive(move || query.with(|p| p.as_ref().ok().and_then(|params| params.sport_id)))
    }
}

impl ParamQueryId for SportIdQuery {
    fn get_id(&self) -> Option<Uuid> {
        self.sport_id
    }
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportConfigIdQuery {
    pub sport_config_id: Option<Uuid>,
}

impl ParamQuery<Uuid> for SportConfigIdQuery {
    fn key() -> &'static str {
        "sport_config_id"
    }
    fn use_param_query() -> Signal<Option<Uuid>> {
        let query = use_query::<Self>();
        Signal::derive(move || {
            query.with(|p| p.as_ref().ok().and_then(|params| params.sport_config_id))
        })
    }
}

impl ParamQueryId for SportConfigIdQuery {
    fn get_id(&self) -> Option<Uuid> {
        self.sport_config_id
    }
}

// ---------------------- Tournament Editor ----------------------

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct TournamentBaseIdQuery {
    pub tournament_id: Option<Uuid>,
}

impl ParamQuery<Uuid> for TournamentBaseIdQuery {
    fn key() -> &'static str {
        "tournament_id"
    }
    fn use_param_query() -> Signal<Option<Uuid>> {
        let query = use_query::<Self>();
        Signal::derive(move || {
            query.with(|p| p.as_ref().ok().and_then(|params| params.tournament_id))
        })
    }
}

impl ParamQueryId for TournamentBaseIdQuery {
    fn get_id(&self) -> Option<Uuid> {
        self.tournament_id
    }
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct StageNumberParams {
    pub stage_number: Option<u32>,
}

impl ParamQuery<u32> for StageNumberParams {
    fn key() -> &'static str {
        "stage_number"
    }
    fn use_param_query() -> Signal<Option<u32>> {
        let query = use_params::<Self>();
        Signal::derive(move || {
            query.with(|p| p.as_ref().ok().and_then(|params| params.stage_number))
        })
    }
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct GroupNumberParams {
    pub group_number: Option<u32>,
}

impl ParamQuery<u32> for GroupNumberParams {
    fn key() -> &'static str {
        "group_number"
    }
    fn use_param_query() -> Signal<Option<u32>> {
        let query = use_params::<Self>();
        Signal::derive(move || {
            query.with(|p| p.as_ref().ok().and_then(|params| params.group_number))
        })
    }
}
