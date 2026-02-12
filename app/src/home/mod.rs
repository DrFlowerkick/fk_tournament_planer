//! home page module

mod dashboard;
mod select_sport;
mod sub_pages;

pub use sub_pages::*;

use crate::home::dashboard::SportDashboard;
use crate::home::select_sport::SelectSportPlugin;
use app_utils::{
    params::{ParamQuery, SportIdQuery},
    state::{
        global_state::{GlobalState, GlobalStateStoreFields},
        toast_state::ToastContext,
    },
};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query, use_url},
    nested_router::Outlet,
};
use reactive_stores::Store;

/// Renders the home page of fk tournament
#[component]
pub fn HomePage() -> impl IntoView {
    // get global state and sport plugin manager
    let toast_context = expect_context::<ToastContext>();

    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    // setup hooks
    let navigate = use_navigate();
    let url = use_url();

    // get query params
    let sport_id_query = use_query::<SportIdQuery>();
    let sport_id = SportIdQuery::use_param_query();

    // check if a sport is active
    let is_sport_active = move || {
        if let Some(sport_id) = sport_id.get()
            && sport_plugin_manager.get().get_web_ui(&sport_id).is_some()
        {
            true
        } else {
            false
        }
    };

    let is_sport_id_given = move || sport_id.get().is_some();

    let is_sport_id_invalid = move || match sport_id_query.get() {
        Ok(sport_params) => {
            if let Some(sport_id) = sport_params.sport_id {
                sport_plugin_manager.get().get_web_ui(&sport_id).is_none()
            } else {
                false
            }
        }
        Err(_) => true,
    };

    Effect::new(move || {
        if is_sport_id_invalid() {
            toast_context.error("Invalid sport id");
            navigate(
                "/",
                NavigateOptions {
                    resolve: false,
                    ..NavigateOptions::default()
                },
            );
        } else if url.get().path() != "/" && !is_sport_id_given() {
            toast_context.error("Missing sport id");
            navigate(
                "/",
                NavigateOptions {
                    resolve: false,
                    ..NavigateOptions::default()
                },
            );
        }
    });

    view! {
        <div class="flex flex-col">
            <Show
                when=move || is_sport_active()
                fallback=|| {
                    view! {
                        <div class="hero py-10 bg-base-100" data-testid="home-hero">
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
                        <div class="px-4">
                            <SelectSportPlugin />
                        </div>
                    }
                }
            >
                <SportDashboard />
            </Show>
        </div>
        <div class="my-4"></div>
        <Outlet />
    }
}
