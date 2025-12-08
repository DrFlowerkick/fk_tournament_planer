//! Sport Config Module

mod edit;
mod search;
mod select_sport;

use app_utils::params::SportParams;
pub use edit::SportConfigForm;
use leptos::prelude::*;
use leptos_router::hooks::use_query;
pub use search::SearchSportConfig;
pub use select_sport::SelectSportPlugin;

#[component]
pub fn SportConfigPage() -> impl IntoView {
    let sport_id_query = use_query::<SportParams>();

    view! {
        <div>
            <SelectSportPlugin />
        </div>
        <div class="my-4"></div>
        <div>
            {move || {
                if let Ok(sport_params) = sport_id_query.get() && sport_params.sport_id.is_some() {
                    view! { <SearchSportConfig /> }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}
