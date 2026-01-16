// web app ui

pub mod home;
pub mod postal_addresses;
pub mod sport_config;

use app_utils::state::global_state::GlobalState;
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

pub fn provide_global_state() {
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
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    // Provides the WebSocket socket context for client registry communication
    provide_socket_context();
    // provide global state context
    provide_global_state();

    view! {
        <Stylesheet id="leptos" href="/pkg/fk_tournament_planer.css" />

        // sets the document title
        <Title text="FK Tournament Planer" />

        // routing
        <Router>
            <div class="flex flex-col min-h-screen">
                // navigation
                <header class="navbar bg-base-200">
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

                <main class="flex-grow p-4">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <ParentRoute path=path!("/") view=HomePage>
                            <Route
                                path=path!("")
                                view={
                                    view! {}
                                }
                            />
                            <Route path=path!("tournaments") view=ListTournaments />
                            <Route path=path!("new-tournament") view=EditTournament />
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
                            <Route path=path!("new_pa") view=PostalAddressForm />
                            <Route path=path!("edit_pa") view=PostalAddressForm />
                        </ParentRoute>
                        <ParentRoute path=path!("/sport") view=SportConfigPage>
                            <Route
                                path=path!("")
                                view={
                                    view! {}
                                }
                            />
                            <Route path=path!("new_sc") view=SportConfigForm />
                            <Route path=path!("edit_sc") view=SportConfigForm />
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
