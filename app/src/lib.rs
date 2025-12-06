// web app ui

pub mod components;
mod error;
pub mod global_state;
pub mod hooks;
pub mod postal_addresses;
pub mod sport_config;

use error::*;
use global_state::GlobalState;
use leptos::prelude::*;
use leptos_axum_socket::provide_socket_context;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{A, ParentRoute, Route, Router, Routes},
    path,
};
use postal_addresses::*;
use reactive_stores::Store;
use sport_config::*;

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
    provide_context(Store::new(GlobalState::new()));

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
                        <a href="/" class="btn btn-ghost normal-case text-xl">
                            "Tournament Planner"
                        </a>
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
                        <Route path=StaticSegment("/") view=HomePage />
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

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <div class="hero min-h-fit bg-base-100">
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">"Welcome!"</h1>
                    <p class="py-6">
                        "This is the development release of the FK Tournament Planner. The application is under active development."
                    </p>
                    <button class="btn btn-primary" on:click=on_click>
                        "Click me: "
                        {count}
                    </button>
                </div>
            </div>
        </div>
    }
}
