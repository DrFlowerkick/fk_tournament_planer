//! Edit tournament group component

use app_utils::hooks::{
    use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    use_scroll_into_view::use_scroll_h2_into_view,
};
use leptos::{html::H2, prelude::*};
use leptos_router::nested_router::Outlet;

#[component]
pub fn EditTournamentGroup() -> impl IntoView {
    let UseQueryNavigationReturn {
        url_is_matched_route,
        ..
    } = use_query_navigation();

    // scroll into view handling
    let scroll_ref = NodeRef::<H2>::new();
    use_scroll_h2_into_view(scroll_ref, url_is_matched_route);

    view! {
        <div class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6">
            <h2 class="text-3xl font-bold" data-testid="group-editor-title" node_ref=scroll_ref>
                "Edit Tournament Group"
            </h2>
            <p class="text-base-content/70 text-center">
                "ToDo: Add information about editing a tournament group."
            </p>
        </div>
        <Outlet />
    }
}
