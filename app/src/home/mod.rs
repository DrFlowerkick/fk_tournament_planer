//! home page module

mod select_sport;

use crate::home::select_sport::SelectSportPlugin;
use app_utils::{
    global_state::{GlobalState, GlobalStateStoreFields},
    params::SportParams,
};
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use reactive_stores::Store;

/// Renders the home page of fk tournament
#[component]
pub fn HomePage() -> impl IntoView {
    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();

    // get query params and helper functions
    let sport_id_query = use_query::<SportParams>();
    let sport_id = move || {
        if let Ok(sport_params) = sport_id_query.get()
            && let Some(sport_id) = sport_params.sport_id
            && sport_plugin_manager.get().get_web_ui(&sport_id).is_some()
        {
            Some(sport_id)
        } else {
            None
        }
    };

    view! {
        {move || {
            match sport_id() {
                Some(_id) => {
                    // ToDo: render sport plugin view
                    view! {
                        <div class="p-4">"Sport View Placeholder (TODO: Render Plugin View)"</div>
                    }
                        .into_any()
                }
                None => {
                    // No sport ID: show hero + grid selection
                    view! {
                        <div class="flex flex-col w-full min-h-screen bg-base-100">
                            // 1. hero / description area
                            <div class="hero py-10" data-testid="home-hero">
                                <div class="hero-content text-center">
                                    <div class="max-w-md">
                                        <h1
                                            class="text-5xl font-bold"
                                            data-testid="home-hero-title"
                                        >
                                            "Welcome!"
                                        </h1>
                                        <p class="py-6" data-testid="home-hero-desc">
                                            "This is the development release of the FK Tournament Planner. The application is under active development."
                                        </p>
                                    </div>
                                </div>
                            </div>

                            // 2. sport plugin selection grid
                            <div class="flex-grow w-full px-4">
                                <SelectSportPlugin />
                            </div>
                        </div>
                    }
                        .into_any()
                }
            }
        }}
    }
}
