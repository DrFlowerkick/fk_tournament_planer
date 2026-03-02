//! web ui for adding and modifying sport configurations

pub mod edit_sport_config;
pub mod list_sport_configs;

pub use edit_sport_config::*;
pub use list_sport_configs::*;

use app_utils::params::{EditActionParams, ParamQuery};
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
pub fn SportConfigRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("sport-configurations") view=ListSportConfigurations>
            <Route
                path=path!("")
                view={
                    view! {}
                }
            />
            <Route path=ParamSegment(EditActionParams::KEY) view=EditSportConfiguration />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}
