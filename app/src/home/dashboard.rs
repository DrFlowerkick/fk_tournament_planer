use app_utils::{
    hooks::use_query_navigation::{
        MatchedRouteHandler, UseQueryNavigationReturn, use_query_navigation,
    },
    params::{ParamQuery, SportIdQuery},
    state::global_state::{GlobalState, GlobalStateStoreFields},
};
use leptos::prelude::*;
use leptos_router::components::A;
use reactive_stores::Store;

#[component]
pub fn SportDashboard() -> impl IntoView {
    // get query helpers
    let UseQueryNavigationReturn {
        url_matched_route,
        url_matched_route_remove_query,
        ..
    } = use_query_navigation();

    // get global state and sport plugin manager
    let state = expect_context::<Store<GlobalState>>();
    let sport_plugin_manager = state.sport_plugin_manager();
    let sport_id = SportIdQuery::use_param_query();

    // Helper to get both ID and Plugin for the view
    let sport_name = move || {
        if let Some(sport_id) = sport_id.get() {
            sport_plugin_manager
                .get()
                .get_web_ui(&sport_id)
                .map(|plugin| plugin.name().to_string())
        } else {
            None
        }
    };

    view! {
        <Show
            when=move || sport_id.get().is_some() && sport_name().is_some()
            fallback=|| {
                view! {
                    <div>
                        "Invalid Sport Plugin ID. This message should never appear, because the parent
                        component should handle this case."
                    </div>
                }
            }
        >
            <div class="card w-full bg-base-100 shadow-xl" data-testid="sport-dashboard">
                <div class="card-body">
                    // Header Section
                    <div class="text-center mb-8 max-w-2xl mx-auto">
                        <A
                            href=move || {
                                format!("/?sport_id={}", sport_id.get().unwrap_or_default())
                            }
                            attr:class="no-underline text-inherit"
                        >
                            <h1
                                class="card-title text-4xl md:text-5xl font-bold mb-4 hover:opacity-80 transition-opacity"
                                data-testid="sport-dashboard-title"
                            >
                                {move || {
                                    format!(
                                        "{} Tournament Planer",
                                        sport_name().unwrap_or_default(),
                                    )
                                }}
                            </h1>
                        </A>
                        <p class="text-xl text-base-content/70" data-testid="sport-dashboard-desc">
                            {move || {
                                format!(
                                    "Welcome to the {} dashboard. Manage tournaments, configure rules, or start a quick game.",
                                    sport_name().unwrap_or_default(),
                                )
                            }}
                        </p>
                    </div>

                    // Navigation Links Grid
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 w-full max-w-3xl mx-auto">
                        <A
                            href=url_matched_route(MatchedRouteHandler::Extend("tournaments"))
                            attr:class="btn btn-primary h-auto min-h-[4rem] text-lg shadow-md"
                            attr:data-testid="link-nav-tournaments"
                            scroll=false
                        >
                            <span class="icon-[heroicons--trophy] w-6 h-6 mr-2"></span>
                            "Tournaments"
                        </A>

                        <A
                            href=url_matched_route_remove_query(
                                "tournament_id",
                                MatchedRouteHandler::Extend("new-tournament"),
                            )

                            attr:class="btn btn-secondary h-auto min-h-[4rem] text-lg shadow-md"
                            attr:data-testid="link-nav-plan-new"
                            scroll=false
                        >
                            <span class="icon-[heroicons--plus-circle] w-6 h-6 mr-2"></span>
                            "Plan New Tournament"
                        </A>

                        <A
                            href=url_matched_route(MatchedRouteHandler::Extend("adhoc-tournament"))
                            attr:class="btn btn-accent h-auto min-h-[4rem] text-lg shadow-md"
                            attr:data-testid="link-nav-adhoc"
                            scroll=false
                        >
                            <span class="icon-[heroicons--play] w-6 h-6 mr-2"></span>
                            "Start Adhoc Tournament"
                        </A>

                        <A
                            href=url_matched_route(
                                MatchedRouteHandler::Extend("sport-configurations"),
                            )
                            attr:class="btn btn-neutral h-auto min-h-[4rem] text-lg shadow-md"
                            attr:data-testid="link-nav-config"
                            scroll=false
                        >
                            <span class="icon-[heroicons--cog-6-tooth] w-6 h-6 mr-2"></span>
                            "Configurations"
                        </A>

                        // Full width About link
                        <A
                            href=url_matched_route(MatchedRouteHandler::Extend("about-sport"))
                            attr:class="btn btn-ghost md:col-span-2 mt-4"
                            attr:data-testid="link-nav-about"
                            scroll=false
                        >
                            {format!("About {}", sport_name().unwrap_or_default())}
                        </A>
                    </div>
                </div>
            </div>
        </Show>
    }
}
