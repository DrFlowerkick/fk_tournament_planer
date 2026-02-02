//! Parameters module for shared query parameter definitions and utilities.

use leptos::prelude::*;
use leptos_router::{
    hooks::{use_params, use_query},
    params::Params,
};
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct AddressIdQuery {
    pub address_id: Option<Uuid>,
}

pub fn use_address_id_query() -> Signal<Option<Uuid>> {
    let query = use_query::<AddressIdQuery>();
    Signal::derive(move || query.with(|p| p.as_ref().ok().and_then(|params| params.address_id)))
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportIdQuery {
    pub sport_id: Option<Uuid>,
}

pub fn use_sport_id_query() -> Signal<Option<Uuid>> {
    let query = use_query::<SportIdQuery>();
    Signal::derive(move || query.with(|p| p.as_ref().ok().and_then(|params| params.sport_id)))
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportConfigIdQuery {
    pub sport_config_id: Option<Uuid>,
}

pub fn use_sport_config_id_query() -> Signal<Option<Uuid>> {
    let query = use_query::<SportConfigIdQuery>();
    Signal::derive(move || {
        query.with(|p| p.as_ref().ok().and_then(|params| params.sport_config_id))
    })
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct TournamentBaseIdQuery {
    pub tournament_id: Option<Uuid>,
}

pub fn use_tournament_base_id_query() -> Signal<Option<Uuid>> {
    let query = use_query::<TournamentBaseIdQuery>();
    Signal::derive(move || query.with(|p| p.as_ref().ok().and_then(|params| params.tournament_id)))
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct StageNumberParams {
    pub stage_number: Option<u32>,
}

pub fn use_stage_number_params() -> Signal<Option<u32>> {
    let params = use_params::<StageNumberParams>();
    Signal::derive(move || params.with(|p| p.as_ref().ok().and_then(|params| params.stage_number)))
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct GroupNumberParams {
    pub group_number: Option<u32>,
}

pub fn use_group_number_params() -> Signal<Option<u32>> {
    let params = use_params::<GroupNumberParams>();
    Signal::derive(move || params.with(|p| p.as_ref().ok().and_then(|params| params.group_number)))
}
