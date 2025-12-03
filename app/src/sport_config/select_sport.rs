//! Component for selecting a sport plugin in the sport configuration flow.

use super::SportParams;
use crate::{
    components::set_id_in_query_input_dropdown::{
        SetIdInQueryInputDropdown, SetIdInQueryInputDropdownProperties,
    },
    sport_config::server_fn::list_sport_plugins,
};
use app_core::utils::id_version::VersionId;
use leptos::prelude::*;
use leptos_router::hooks::use_query;

use leptos::logging::log;

#[component]
pub fn SelectSportPlugin() -> impl IntoView {
    let plugins = OnceResource::new(list_sport_plugins());
    let sport_id_query = use_query::<SportParams>();
    let name = RwSignal::new("".to_string());
    let search_text = RwSignal::new("".to_string());

    let sport_list = Signal::derive(move || {
        if let Some(Ok(sport_plugins)) = plugins.get() {
            let search_text_lower = search_text.read().to_lowercase();
            sport_plugins
                .into_iter()
                .filter(|spi| {
                    if search_text_lower.is_empty() {
                        true
                    } else {
                        spi.get_name().to_lowercase().contains(&search_text_lower)
                    }
                })
                .collect()
        } else {
            vec![]
        }
    });

    Effect::new(move || {
        if let Ok(sport_params) = sport_id_query.get()
            && let Some(sport_id) = sport_params.sport_id
        {
            let sport_name = sport_list
                .get_untracked()
                .iter()
                .find(|spi| spi.get_id_version().get_id() == Some(sport_id))
                .map(|spi| spi.get_name())
                .unwrap_or_default();
            log!("Selected sport name: {}", sport_name);
            name.set(sport_name);
        } else {
            name.set("".to_string());
        }
    });

    let set_id_in_query_input_dropdown_props = SetIdInQueryInputDropdownProperties {
        key: "sport_id",
        name,
        placeholder: "Enter name of address you are searching...",
        search_text: search_text,
        list_items: sport_list,
        render_item: |spi| view! { <span class="font-medium">{spi.get_name()}</span> }.into_any(),
    };

    view! {
        <div class="card w-full bg-base-100 shadow-xl">
            <div class="card-body">
                <h2 class="card-title">"Search Sport"</h2>
                <Transition fallback=move || {
                    view! {
                        <div class="flex justify-center items-center p-4">
                            <span class="loading loading-spinner loading-lg"></span>
                        </div>
                    }
                }>
                    <SetIdInQueryInputDropdown
                        props=set_id_in_query_input_dropdown_props
                    />
                </Transition>
            </div>
        </div>
        /*<div class="my-4"></div>
        {if cfg!(not(feature = "test-mock")) {
            view! { <Outlet /> }.into_any()
        } else {
            ().into_any()
        }}*/
    }
}
