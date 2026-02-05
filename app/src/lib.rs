#![recursion_limit = "512"]
// web app ui

pub mod home;
pub mod postal_addresses;
pub mod sport_config;

use app_utils::{
    components::{banner::GlobalErrorBanner, toast::ToastContainer},
    state::{error_state::PageErrorContext, global_state::GlobalState, toast_state::ToastContext},
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
use sport_config::*;
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
            // global error banner and toast container are placed here so they are available on all pages
            <GlobalErrorBanner />
            <ToastContainer />
            <div class="flex flex-col min-h-screen">
                // navigation
                <header class="navbar bg-base-300">
                    <div class="flex-1">
                        <A href="/" attr:class="btn btn-ghost normal-case text-xl">
                            "Tournament Planner"
                        </A>
                    </div>
                    <div class="flex-none">
                        <ul class="menu menu-horizontal px-1">
                            <li>
                                <A href="/postal-address">"Postal Addresses"</A>
                            </li>
                            <li>
                                <A href="/sport">"Sports"</A>
                            </li>
                        </ul>
                    </div>
                </header>

                <main class="flex-grow p-4 bg-base-200">
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
                            <Route path=path!("sport-configurations") view=SportConfigurations />
                            <Route path=path!("about-sport") view=AboutSport />

                        </ParentRoute>
                        <ParentRoute path=path!("/postal-address") view=SearchPostalAddress>
                            <Route
                                path=path!("")
                                view={
                                    view! {}
                                }
                            />
                            <Route path=path!("new_pa") view=LoadPostalAddress />
                            <Route path=path!("edit_pa") view=LoadPostalAddress />
                        </ParentRoute>
                        <ParentRoute path=path!("/sport") view=SportConfigPage>
                            <Route
                                path=path!("")
                                view={
                                    view! {}
                                }
                            />
                            <Route path=path!("new_sc") view=LoadSportConfig />
                            <Route path=path!("edit_sc") view=LoadSportConfig />
                        </ParentRoute>
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
