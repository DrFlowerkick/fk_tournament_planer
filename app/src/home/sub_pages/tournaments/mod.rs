//! listing tournaments

pub mod list_tournaments;
pub mod register_at_tournament;

pub use list_tournaments::*;
pub use register_at_tournament::*;

use crate::{EditSubRoutes, EditTournament};
use leptos::prelude::*;
#[allow(unused_imports)]
use leptos_router::MatchNestedRoutes;
use leptos_router::{
    any_nested_route::IntoAnyNestedRoute,
    components::{ParentRoute, Route},
    path,
};

#[component(transparent)]
pub fn TournamentsRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("tournaments") view=ListTournaments>
            <ParentRoute path=path!("edit") view=EditTournament>
                <EditSubRoutes />
                <Route
                    path=path!("")
                    view={
                        view! {}
                    }
                />
            </ParentRoute>
            <Route
                path=path!("")
                view={
                    view! {}
                }
            />
            <Route path=path!("register") view=RegisterAtTournament />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}
