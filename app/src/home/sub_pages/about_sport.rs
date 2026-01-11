//! displaying information about a sport plugin

use leptos::prelude::*;

#[component]
pub fn AboutSport() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6">
            <h2 class="text-3xl font-bold">"About This Sport"</h2>
            <p class="text-base-content/70 text-center">
                "ToDo: Add information about the selected sport plugin here, such as its features, rules, and how to use it within the FK Tournament Planner."
            </p>
        </div>
    }
}
