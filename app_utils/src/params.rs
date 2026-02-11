//! Parameters module for shared query parameter definitions and utilities.

use crate::enum_utils::FilterLimit;
use leptos::prelude::*;
use leptos_router::{
    hooks::{use_params, use_query},
    params::Params,
};
use uuid::Uuid;

pub trait ParamQuery<T: Send + Sync + 'static>: Params {
    fn key() -> &'static str;
    fn use_param_query() -> Signal<Option<T>>;
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
