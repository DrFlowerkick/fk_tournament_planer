//! Component for selecting a sport plugin in the sport configuration flow.

use super::SportParams;
use crate::{
    components::{
        banner::AcknowledgmentBanner,
        set_id_in_query_input_dropdown::{
            SetIdInQueryInputDropdown, SetIdInQueryInputDropdownProperties,
        },
    },
    global_state::{GlobalState, GlobalStateStoreFields},
};
use app_core::{SportPluginManagerPort, SportPort, utils::id_version::VersionId};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_navigate, use_query},
};
use reactive_stores::Store;
use std::sync::Arc;

#[component]
pub fn SelectSportPlugin() -> impl IntoView {
    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();

    let sport_id_query = use_query::<SportParams>();

    let is_sport_id_error = move || {
        if let Ok(sport_params) = sport_id_query.get() {
            if let Some(sport_id) = sport_params.sport_id {
                sport_plugin_manager.get().get_web_ui(&sport_id).is_none()
            } else {
                false
            }
        } else {
            true
        }
    };

    // reset url when unexpectedly no sport found
    // ToDo: this has to be expanded, when SelectSportPlugin is used in other places
    let reset_url = move || {
        let navigate = use_navigate();
        navigate(
            "/sport",
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
    };

    let name = RwSignal::new("".to_string());
    let search_text = RwSignal::new("".to_string());

    let sport_list: Signal<Vec<Arc<dyn SportPort>>> = Signal::derive(move || {
        let search_text_lower = search_text.read().to_lowercase();
        sport_plugin_manager
            .get()
            .list()
            .into_iter()
            .filter(|sp| {
                if search_text_lower.is_empty() {
                    true
                } else {
                    sp.name().to_lowercase().contains(&search_text_lower)
                }
            })
            .collect()
    });

    Effect::new(move || {
        if let Ok(sport_params) = sport_id_query.get()
            && let Some(sport_id) = sport_params.sport_id
        {
            let sport_name = sport_list
                .get_untracked()
                .iter()
                .find(|spi| spi.get_id_version().get_id() == Some(sport_id))
                .map(|spi| spi.name())
                .unwrap_or_default();
            name.set(sport_name.to_string());
        } else {
            name.set("".to_string());
        }
    });

    let props = SetIdInQueryInputDropdownProperties {
        key: "sport_id",
        name,
        placeholder: "Enter name of sport you are searching...",
        search_text: search_text,
        list_items: sport_list,
        render_item: |spi| view! { <span class="font-medium">{spi.name()}</span> }.into_any(),
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
                    {move || {
                        let props = props.clone();
                        if is_sport_id_error() {
                            view! {
                                <AcknowledgmentBanner
                                    msg="The selected sport id could not be found. Please select a valid sport."
                                    ack_btn_text="Reset ID"
                                    ack_action=reset_url
                                />
                            }
                                .into_any()
                        } else {
                            view! { <SetIdInQueryInputDropdown props=props /> }.into_any()
                        }
                    }}
                </Transition>
            </div>
        </div>
    }
}
