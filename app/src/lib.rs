// web app ui

mod banner;
mod error;
mod postal_addresses;
use error::*;
use postal_addresses::*;

use leptos::prelude::*;
use leptos_axum_socket::provide_socket_context;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{A, Route, Router, Routes},
    path,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
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
    #[cfg(feature = "hydrate")]
    {
        provide_socket_context();
    }

    view! {
        <Stylesheet id="leptos" href="/pkg/fk_tournament_planer.css" />

        // sets the document title
        <Title text="Welcome to FK Tournament Planer" />

        // routing
        <Router>

            // navigation
            <nav>
                <A href="/">"Home"</A>
                <A href="/postal-address">"Postal Address"</A>
            </nav>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("/") view=HomePage />
                    <Route path=path!("/postal-address") view=SearchPostalAddress />
                    <Route path=path!("/postal-address/new") view=NewPostalAddress />
                    <Route path=path!("/postal-address/:uuid/edit") view=PostalAddressEdit />
                    <Route path=path!("/postal-address/:uuid") view=SearchPostalAddress />
                </Routes>
            </main>
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
        <h1>"Welcome to FK Tournament Planer!"</h1>
        <button class="btn" on:click=on_click>
            "Click Me: "
            {count}
        </button>
    }
}
