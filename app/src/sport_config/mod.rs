//! Sport Config Module

mod search;
mod select_sport;
pub mod server_fn;

use leptos::{Params, prelude::*};
use leptos_router::{hooks::use_query, params::Params};
pub use search::SearchSportConfig;
pub use select_sport::SelectSportPlugin;
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportParams {
    pub sport_id: Option<Uuid>,
}

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportConfigParams {
    pub sport_config_id: Option<Uuid>,
}

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
