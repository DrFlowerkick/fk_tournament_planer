#![recursion_limit = "512"]
// web app ui

pub mod home;
pub mod postal_addresses;

use app_utils::{
    components::{global_error_banner::GlobalErrorBanner, toast::ToastContainer},
    hooks::blur_active_element::blur_active_element,
    state::{
        activity_tracker::ActivityTracker, error_state::PageErrorContext,
        global_state::GlobalState, toast_state::ToastContext,
    },
};
use ddc_plugin::DdcSportPlugin;
use generic_sport_plugin::GenericSportPlugin;
use home::*;
use leptos::prelude::*;
use leptos_axum_socket::provide_socket_context;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{A, ParentRoute, Route, Router, Routes},
    path,
};
use postal_addresses::*;
use reactive_stores::Store;
use std::sync::Arc;

pub fn provide_global_context() {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    // Provides the WebSocket socket context for client registry communication
    provide_socket_context();
    // set context for error reporting
    let page_error_context = PageErrorContext::new();
    provide_context(page_error_context);
    let toast_context = ToastContext::new();
    provide_context(toast_context);
    // set context for global activity tracker
    let activity_tracker = ActivityTracker::new();
    provide_context(activity_tracker);

    let mut global_state = GlobalState::new();
    global_state
        .sport_plugin_manager
        .register(Arc::new(GenericSportPlugin::new()))
        .unwrap();
    global_state
        .sport_plugin_manager
        .register(Arc::new(DdcSportPlugin::new()))
        .unwrap();
    provide_context(Store::new(global_state));
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="fantasy">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // provide global context elements
    provide_global_context();

    // Get the error context to reactively toggle the inert state
    let page_err_ctx = expect_context::<PageErrorContext>();

    // Get the activity tracker context to reactively toggle the inert state
    let activity_tracker = expect_context::<ActivityTracker>();

    // Signal to manage the mobile menu state
    let (menu_open, set_menu_open) = signal(false);

    // HYDRATION MARKER for E2E TESTS:
    // This effect runs only on the client once the WASM is active and hydration is complete.
    // We mark the body so Playwright knows exactly when it's safe to click.
    Effect::new(|_| {
        if let Some(body) = document().body() {
            let _ = body.set_attribute("data-hydrated", "true");
        }
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/fk_tournament_planer.css" />

        // sets the document title
        <Title text="FK Tournament Planer" />

        // routing
        <Router>
            <div class="flex flex-col min-h-screen">
                // navigation
                <header class="navbar bg-base-300 sticky top-0 z-50">
                    <div class="flex-1">
                        <A href="/" attr:class="btn btn-ghost normal-case text-xl">
                            "Tournament Planner"
                        </A>
                    </div>
                    // Group loading indicator and menu button together on the right
                    <div class="flex-none flex items-center gap-3 px-2">
                        <Show when=move || activity_tracker.is_active.get()>
                            <span class="loading loading-bars loading-sm"></span>
                        </Show>
                        <div class="dropdown dropdown-end" class:dropdown-open=menu_open>
                            // Use a button instead of label/input to avoid event conflicts in Leptos.
                            // The 'swap-active' class controls which icon is visible based on the signal.
                            // Trigger blur if false or when clicking links to ensure closing the dropdown menu.
                            // This is required because daisyUI's dropdown relies on focus/blur of CSS selectors.
                            <button
                                type="button"
                                class="btn btn-ghost btn-circle swap swap-rotate"
                                class:swap-active=menu_open
                                on:click=move |_| {
                                    set_menu_open.update(|v| *v = !*v);
                                    if !menu_open.get() {
                                        blur_active_element();
                                    }
                                }
                                on:blur=move |_| set_menu_open.set(false)
                            >
                                // Hamburger menu icon (visible when menu_open is false)
                                <span class="swap-off icon-[heroicons--bars-3] w-6 h-6 inline-block"></span>

                                // Close icon (visible when menu_open is true)
                                <span class="swap-on icon-[heroicons--x-mark] w-6 h-6 inline-block"></span>
                            </button>

                            // Vertical dropdown menu
                            <ul
                                tabindex="0"
                                class="dropdown-content menu bg-base-100 rounded-box z-[1] mt-3 w-52 p-2 shadow border border-base-content/10"
                            >
                                <li>
                                    <A
                                        href="/postal-address"
                                        on:click=move |_| {
                                            set_menu_open.set(false);
                                            blur_active_element();
                                        }
                                    >
                                        "Postal Addresses"
                                    </A>
                                </li>
                            </ul>
                        </div>
                    </div>
                </header>
                // global toast container is placed here so they are available on all pages
                <ToastContainer />
                // global error banner is placed here to be always on top of the page content, but below the navbar
                <div class="sticky z-40 top-16 bg-base-200">
                    <GlobalErrorBanner />
                </div>
                <main
                    class="flex-grow p-4 bg-base-200 transition-all duration-200"
                    class:opacity-50=move || page_err_ctx.has_errors()
                    inert=move || page_err_ctx.has_errors()
                >
                    <Routes fallback=|| "Page not found.".into_view()>
                        <ParentRoute path=path!("/") view=HomePage>
                            <Route
                                path=path!("")
                                view={
                                    view! {}
                                }
                            />
                            <TournamentsRoutes />
                            <NewTournamentRoutes />
                            <Route path=path!("adhoc-tournament") view=AdhocTournament />
                            <SportConfigRoutes />
                            <Route path=path!("about-sport") view=AboutSport />
                        </ParentRoute>
                        <PostalAddressRoutes />
                    </Routes>
                </main>

                <footer class="footer footer-center p-4 bg-base-300 text-base-content">
                    <div>
                        <p>"© 2025 FK-Tournament-Planer - Alle Rechte vorbehalten"</p>
                    </div>
                </footer>
            </div>
        </Router>
    }
}
