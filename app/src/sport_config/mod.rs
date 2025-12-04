//! Sport Config Module

mod search;
mod select_sport;
pub mod server_fn;

use leptos::{Params, prelude::*};
use leptos_router::{hooks::use_query, params::Params};
use search::SearchSportConfig;
pub use select_sport::SelectSportPlugin;
use uuid::Uuid;

#[derive(Params, Clone, PartialEq, Eq, Debug)]
pub struct SportParams {
    pub sport_id: Option<Uuid>,
}

#[component]
pub fn SportConfigPage() -> impl IntoView {
    let sport_id_query = use_query::<SportParams>();

    view! {
        <div>
            <SelectSportPlugin />
        </div>
        <div class="p-4">
            {move || {
                if let Ok(sport_params) = sport_id_query.get()
                    && let Some(sport_id) = sport_params.sport_id
                {
                    view! { <SearchSportConfig sport_id=sport_id /> }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}
