//! Edit tournament components

pub mod tournament_base;
pub mod tournament_group;
pub mod tournament_stage;

pub use tournament_base::*;
pub use tournament_group::*;
pub use tournament_stage::*;

use app_utils::params::{GroupNumberParams, ParamQuery, StageNumberParams};
use leptos::prelude::*;
#[allow(unused_imports)]
use leptos_router::MatchNestedRoutes;
use leptos_router::{
    ParamSegment,
    any_nested_route::IntoAnyNestedRoute,
    components::{ParentRoute, Route},
    path,
};

#[component(transparent)]
pub fn NewTournamentRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("new-tournament") view=LoadTournament>
            <EditSubRoutes />
            <Route
                path=path!("")
                view={
                    view! {}
                }
            />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}

#[component(transparent)]
pub fn EditSubRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=ParamSegment(StageNumberParams::KEY) view=LoadTournamentStage>
            <Route
                path=path!("")
                view={
                    view! {}
                }
            />
            <Route path=ParamSegment(GroupNumberParams::KEY) view=EditTournamentGroup />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}
