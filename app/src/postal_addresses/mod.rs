// web ui for adding and modifying postal addresses

mod copy_postal_addresses;
mod edit_postal_addresses;
mod list_postal_addresses;

pub use copy_postal_addresses::*;
pub use edit_postal_addresses::*;
pub use list_postal_addresses::*;

use leptos::prelude::*;
#[allow(unused_imports)]
use leptos_router::MatchNestedRoutes;
use leptos_router::{
    any_nested_route::IntoAnyNestedRoute,
    components::{ParentRoute, Route},
    path,
};

#[component(transparent)]
pub fn PostalAddressRoutes() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("/postal-address") view=ListPostalAddresses>
            <Route
                path=path!("")
                view={
                    view! {}
                }
            />
            <Route path=path!("new") view=LoadPostalAddress />
            <Route path=path!("edit") view=LoadPostalAddress />
            <Route path=path!("copy") view=CopyPostalAddress />
        </ParentRoute>
    }
    .into_inner()
    .into_any_nested_route()
}
