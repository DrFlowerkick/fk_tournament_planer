//! Sport Config Edit Module

use app_utils::{
    global_state::{GlobalState, GlobalStateStoreFields},
    params::SportParams,
};
use leptos::{logging::log, prelude::*};
use leptos_router::hooks::use_query;
use reactive_stores::Store;

#[component]
pub fn SportConfigForm() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let sport_query = use_query::<SportParams>();

    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();

    let sport_plugin = move || {
        if let Ok(sport_params) = sport_query.get()
            && let Some(sport_id) = sport_params.sport_id
        {
            sport_plugin_manager.get().get_web_ui(&sport_id)
        } else {
            log!("No valid sport_id in query params. Searching sport config is disabled.");
            None
        }
    };
    view! { {move || sport_plugin().map(|plugin| plugin.render_configuration())} }
}
