//! Edit tournament group component

use leptos::prelude::*;
use leptos_router::nested_router::Outlet;

#[component]
pub fn EditTournamentGroup() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center w-full max-w-4xl mx-auto py-8 space-y-6">
            <h2 class="text-3xl font-bold">"Edit Tournament Group"</h2>
            <p class="text-base-content/70 text-center">
                "ToDo: Add information about editing a tournament group."
            </p>
        </div>
        <Outlet />
    }
}
