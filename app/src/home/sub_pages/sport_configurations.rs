//! listing, creating and modifying sport configurations
/*
use app_core::{TournamentBase, TournamentState, TournamentType, SportConfig};
use app_utils::{
    components::inputs::EnumSelectWithValidation,
    error::{
        AppError,
        strategy::{handle_general_error, handle_read_error},
    },
    hooks::{
        use_on_cancel::use_on_cancel,
        use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    },
    params::use_sport_id_query,
    server_fn::tournament_base::list_tournament_bases,
    state::error_state::PageErrorContext,
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, components::A, hooks::use_navigate, nested_router::Outlet};
use uuid::Uuid;
*/
use leptos::prelude::*;

#[component]
pub fn SportConfigurations() -> impl IntoView {
    /*
    // navigation and query handling Hook
    let UseQueryNavigationReturn {
        url_with_update_query,
        url_with_remove_query,
        ..
    } = use_query_navigation();
    let navigate = use_navigate();

    // --- global context ---
    let page_err_ctx = expect_context::<PageErrorContext>();
    let component_id = StoredValue::new(Uuid::new_v4());
    // remove errors on unmount
    on_cleanup(move || {
        page_err_ctx.clear_all_for_component(component_id.get_value());
    });

    // Signals for Filters
    let (search_term, set_search_term) = signal("".to_string());
    let (limit, set_limit) = signal(10usize);
    */
    view! {
        <div class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6">
            <h2 class="text-3xl font-bold">"Sport Configurations"</h2>
            <p class="text-base-content/70 text-center">
                "ToDo: Add information about listing, creating, and modifying sport configurations here, such as its features, rules, and how to use it within the FK Tournament Planner."
            </p>
        </div>
    }
}
