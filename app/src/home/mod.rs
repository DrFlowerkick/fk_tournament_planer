//! home page module

mod select_sport;

use leptos::prelude::*;

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
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