//!

pub mod copy_sport_config;
pub mod edit_sport_config;
pub mod list_sport_configs;

pub use copy_sport_config::*;
pub use edit_sport_config::*;
pub use list_sport_configs::*;

use leptos::prelude::*;
#[allow(unused_imports)]
use leptos_router::MatchNestedRoutes;
use leptos_router::{
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
            <Route path=path!("new") view=LoadSportConfiguration />
            <Route path=path!("edit") view=LoadSportConfiguration />
            <Route path=path!("copy") view=CopySportConfiguration />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}
