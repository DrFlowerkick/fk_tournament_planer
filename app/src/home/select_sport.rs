//! Component for selecting a sport plugin in the sport configuration flow.

use app_core::{SportPluginManagerPort, utils::traits::ObjectIdVersion};
use app_utils::{
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    state::global_state::{GlobalState, GlobalStateStoreFields},
};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use reactive_stores::Store;

#[component]
pub fn SelectSportPlugin() -> impl IntoView {
    // get query helpers
    let UseQueryNavigationReturn {
        nav_url, update, ..
    } = use_query_navigation();

    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();

    let sport_list = Signal::derive(move || {
        let mut list = sport_plugin_manager.get().list();
        // Sort plugins alphabetically by name to ensure stable order
        list.sort_by_key(|plugin| plugin.name());
        list
    });

    let (selected_sport_id, set_selected_sport_id) = signal::<Option<uuid::Uuid>>(None);

    Effect::new(move || {
        if let Some(sport_id) = selected_sport_id.get() {
            update("sport_id", &sport_id.to_string());
            let navigate = use_navigate();
            navigate(
                &nav_url.get(),
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    view! {
        <div class="flex flex-col items-center w-full max-w-6xl mx-auto space-y-8 py-4">
            <div class="text-center mb-4">
                <h2 class="text-3xl font-bold">"Select a Sport"</h2>
                <p class="text-base-content/70 mt-2">
                    "Choose a sport plugin below to start planning your tournament."
                </p>
            </div>

            <div
                class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 w-full bg-transparent"
                data-testid="sport-selection-grid"
            >
                <For
                    each=move || sport_list.get()
                    key=|plugin| plugin.get_id_version().get_id().unwrap_or_default()
                    children=move |plugin| {
                        let id = plugin.get_id_version().get_id().unwrap_or_default();
                        let web_ui_plugin = sport_plugin_manager.get().get_web_ui(&id);
                        let plugin_name = plugin.name();
                        let test_id_suffix = plugin_name.replace(" ", "");
                        // Generate a stable test ID from the name (remove whitespace)
                        // e.g. "Double Disc Court (DDC)" -> "DoubleDiscCourt(DDC)"

                        view! {
                            {match web_ui_plugin {
                                Some(ui) => {
                                    view! {
                                        <button
                                            class="btn btn-outline h-auto min-h-[12rem] w-full flex flex-col items-center justify-center p-6 bg-base-100 hover:bg-base-200 hover:border-primary transition-all duration-300 shadow-md hover:shadow-xl rounded-xl border-dashed border-2"
                                            // Stable Test ID derived from name
                                            data-testid=format!("btn-select-sport-{}", test_id_suffix)
                                            // Accessibility label
                                            aria-label=plugin_name
                                            on:click=move |_| set_selected_sport_id.set(Some(id))
                                        >
                                            // The plugin renders its own representation inside our wrapper button
                                            {ui.render_plugin_selection()}
                                        </button>
                                    }
                                        .into_any()
                                }
                                None => {
                                    // Error Card: keeps the dimensions and shows the error clearly
                                    view! {
                                        <div
                                            class="h-auto min-h-[12rem] w-full flex flex-col items-center justify-center p-6 rounded-xl border-2 border-error border-dashed opacity-70 cursor-not-allowed bg-base-100"
                                            title="Plugin UI Implementation missing"
                                        >
                                            <div class="text-error text-4xl mb-2">"⚠️"</div>
                                            <span class="text-error font-bold">"UI Missing"</span>
                                            <span class="text-xs text-center mt-2 font-mono truncate max-w-full px-2">
                                                {format!("ID: {}", id)}
                                            </span>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }}
                        }
                    }
                />
            </div>
        </div>
    }
}
