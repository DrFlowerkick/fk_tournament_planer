//! creating a new tournament

use leptos::prelude::*;

#[component]
pub fn NewTournament() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6">
            <h2 class="text-3xl font-bold">"New Tournament"</h2>
            <p class="text-base-content/70 text-center">
                "ToDo: Add information about creating a new tournament here, such as its features, rules, and how to use it within the FK Tournament Planner."
            </p>
        </div>
    }
}
