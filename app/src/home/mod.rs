//! home page module

mod dashboard;
mod select_sport;
mod sub_pages;

pub use sub_pages::*;

use crate::home::dashboard::SportDashboard;
use crate::home::select_sport::SelectSportPlugin;
use app_utils::{
    components::{banner::GlobalErrorBanner, toast::ToastContainer},
    params::{SportIdQuery, use_sport_id_query},
    state::{
        error_state::PageErrorContext,
        global_state::{GlobalState, GlobalStateStoreFields},
        toast_state::{ToastContext, ToastVariant},
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
    // set context for error reporting
    let page_error_context = PageErrorContext::new();
    provide_context(page_error_context);
    let toast_context = ToastContext::new();
    provide_context(toast_context);
    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    // setup hooks
    let navigate = use_navigate();
    let url = use_url();

    // get query params
    let sport_id_query = use_query::<SportIdQuery>();
    let sport_id = use_sport_id_query();

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
            toast_context.add("Invalid sport id", ToastVariant::Error);
            navigate(
                "/",
                NavigateOptions {
                    resolve: false,
                    ..NavigateOptions::default()
                },
            );
        } else if url.get().path() != "/" && !is_sport_id_given() {
            toast_context.add("Missing sport id", ToastVariant::Error);
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
        <GlobalErrorBanner />
        <ToastContainer />
        <div class="flex flex-col min-h-screen">
            {move || {
                if is_sport_active() {
                    view! { <SportDashboard /> }.into_any()
                } else {
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
                        .into_any()
                }
            }} <div class="flex-grow w-full">
                <Outlet />
            </div>
        </div>
    }
}
