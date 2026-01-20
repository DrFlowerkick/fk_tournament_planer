use app_utils::{
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::SportParams,
    state::global_state::{GlobalState, GlobalStateStoreFields},
};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_query};
use reactive_stores::Store;

#[component]
pub fn SportDashboard() -> impl IntoView {
    // get query helpers
    let UseQueryNavigationReturn {
        relative_sub_url,
        url_with_out_param,
        ..
    } = use_query_navigation();

    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_id_query = use_query::<SportParams>();

    let sport_plugin = move || {
        if let Ok(params) = sport_id_query.get()
            && let Some(sport_id) = params.sport_id
        {
            sport_plugin_manager.get().get_web_ui(&sport_id)
        } else {
            None
        }
    };

    view! {
        {move || {
            if let Some(plugin) = sport_plugin() {
                let name = plugin.name();

                view! {
                    <div
                        class="flex flex-col items-center w-full bg-base-100 p-8 pt-12"
                        data-testid="sport-dashboard"
                    >
                        // Header Section
                        <div class="text-center mb-8 max-w-2xl">
                            <h1
                                class="text-4xl md:text-5xl font-bold mb-4"
                                data-testid="sport-dashboard-title"
                            >
                                {format!("{} Tournament Planer", name)}
                            </h1>
                            <p
                                class="text-xl text-base-content/70"
                                data-testid="sport-dashboard-desc"
                            >
                                {format!(
                                    "Welcome to the {} dashboard. Manage tournaments, configure rules, or start a quick game.",
                                    name,
                                )}
                            </p>
                        </div>

                        // Navigation Links Grid
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full max-w-3xl">
                            <A
                                href=relative_sub_url("/tournaments")
                                attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                                attr:data-testid="link-nav-tournaments"
                                scroll=false
                            >
                                <span class="icon-[heroicons--trophy] w-6 h-6 mr-2"></span>
                                "Tournaments"
                            </A>

                            <A
                                href=url_with_out_param("tournament_id", Some("/new-tournament"))

                                attr:class="btn btn-secondary h-auto min-h-[4rem] text-lg shadow-md"
                                attr:data-testid="link-nav-plan-new"
                                scroll=false
                            >
                                <span class="icon-[heroicons--plus-circle] w-6 h-6 mr-2"></span>
                                "Plan New Tournament"
                            </A>

                            <A
                                href=relative_sub_url("/adhoc-tournament")
                                attr:class="btn btn-accent h-auto min-h-[4rem] text-lg shadow-md"
                                attr:data-testid="link-nav-adhoc"
                                scroll=false
                            >
                                <span class="icon-[heroicons--play] w-6 h-6 mr-2"></span>
                                "Start Adhoc Tournament"
                            </A>

                            <A
                                href=relative_sub_url("/sport-configurations")
                                attr:class="btn btn-neutral h-auto min-h-[4rem] text-lg shadow-md"
                                attr:data-testid="link-nav-config"
                                scroll=false
                            >
                                <span class="icon-[heroicons--cog-6-tooth] w-6 h-6 mr-2"></span>
                                "Configurations"
                            </A>

                            // Full width About link
                            <A
                                href=relative_sub_url("/about-sport")
                                attr:class="btn btn-ghost md:col-span-2 mt-4"
                                attr:data-testid="link-nav-about"
                                scroll=false
                            >
                                {format!("About {}", name)}
                            </A>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                // Fallback for when the ID is invalid (should be caught by the parent, but just in case)
                view! {
                    <div>
                        "Invalid Sport Plugin ID. This message should never appear, because the parent
                        component should handle this case."
                    </div>
                }
                    .into_any()
            }
        }}
    }
}
