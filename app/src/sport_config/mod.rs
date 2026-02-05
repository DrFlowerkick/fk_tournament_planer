//! Sport Config Module

mod edit;
mod search;
mod select_sport;

use app_utils::params::use_sport_id_query;
pub use edit::LoadSportConfig;
use leptos::prelude::*;
use leptos_router::nested_router::Outlet;
pub use search::SearchSportConfig;
pub use select_sport::SelectSportPlugin;

#[component]
pub fn SportConfigPage() -> impl IntoView {
    let sport_id = use_sport_id_query();

    view! {
        <div>
            <SelectSportPlugin />
        </div>
        <div class="my-4"></div>
        <div>
            {move || {
                if sport_id.get().is_some() {
                    view! { <SearchSportConfig /> }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>

        <div class="mt-4">
            <Outlet />
        </div>
    }
}
