//! home page module

mod dashboard;
mod select_sport;
mod sub_pages;

pub use sub_pages::*;

use crate::home::dashboard::SportDashboard;
use crate::home::select_sport::SelectSportPlugin;
use app_utils::{
    state::global_state::{GlobalState, GlobalStateStoreFields},
    params::SportParams,
};
use leptos::prelude::*;
use leptos_router::{hooks::use_query, nested_router::Outlet};
use reactive_stores::Store;

/// Renders the home page of fk tournament
#[component]
pub fn HomePage() -> impl IntoView {
    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();

    // get query params
    let sport_id_query = use_query::<SportParams>();

    // check if a sport is active
    let is_sport_active = move || {
        if let Ok(sport_params) = sport_id_query.get()
            && let Some(sport_id) = sport_params.sport_id
            && sport_plugin_manager.get().get_web_ui(&sport_id).is_some()
        {
            true
        } else {
            false
        }
    };

    view! {
        {move || {
            if is_sport_active() {
                // Dashboard + Outlet
                view! {
                    <div class="flex flex-col min-h-screen">
                        <SportDashboard />
                        <div class="flex-grow w-full">
                            <Outlet />
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                // No/Invalid Sport ID: Show Hero + Selection
                view! {
                    <div class="flex flex-col w-full min-h-screen bg-base-100">
                        <div class="hero py-10" data-testid="home-hero">
                            <div class="hero-content text-center">
                                <div class="max-w-md">
                                    <h1 class="text-5xl font-bold" data-testid="home-hero-title">
                                        "Welcome!"
                                    </h1>
                                    <p class="py-6" data-testid="home-hero-desc">
                                        "This is the development release of the FK Tournament Planner. The application is under active development."
                                    </p>
                                </div>
                            </div>
                        </div>

                        <div class="flex-grow w-full px-4">
                            <SelectSportPlugin />
                        </div>
                    </div>
                }
                    .into_any()
            }
        }}
    }
}
