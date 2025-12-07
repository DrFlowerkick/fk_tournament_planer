//! Implementation of sport preview for the generic sport plugin

use super::GenericSportPlugin;
use app_core::SportConfig;
use app_utils::{
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::{SportConfigParams, SportParams},
};
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use shared::SportConfigWebUi;

impl SportConfigWebUi for GenericSportPlugin {
    fn render_preview(&self, config: &SportConfig) -> AnyView {
        let generic_config = match self.validate_config(config) {
            Ok(cfg) => cfg,
            Err(_) => return view! { <div>{"Invalid Configuration"}</div> }.into_any(),
        };
        view! {
            <div class="card w-full bg-base-200 shadow-md mt-4" data-testid="sport-config-preview">
                <div class="card-body">
                    <h3 class="card-title" data-testid="preview-sport-name">
                        {config.name.clone()}
                    </h3>
                    {move || {
                        generic_config
                            .score_to_win
                            .map(|score| {
                                view! {
                                    <p data-testid="preview-score-config">
                                        <span data-testid="preview-sets-to-win">
                                            {format!("Sets to win: {}", generic_config.sets_to_win)}
                                        </span>
                                        " - "
                                        <span data-testid="preview-score-to-win">
                                            {format!("Score to win a set: {}", score)}
                                        </span>
                                        {move || {
                                            generic_config
                                                .win_by_margin
                                                .map(|margin| {
                                                    view! {
                                                        <span data-testid="preview-win-by-margin">
                                                            {" (win by "} {margin} {")"}
                                                        </span>
                                                    }
                                                })
                                        }}
                                        {move || {
                                            generic_config
                                                .hard_cap
                                                .map(|cap| {
                                                    view! {
                                                        <span data-testid="preview-hard-cap">
                                                            {" (hard cap "} {cap} {")"}
                                                        </span>
                                                    }
                                                })
                                        }}
                                    </p>
                                }
                                    .into_any()
                            })
                    }}
                    <p data-testid="preview-victory-points">
                        {format!(
                            "Victory Points - Win: {}, Draw: {}",
                            generic_config.victory_points_win,
                            generic_config.victory_points_draw,
                        )}
                    </p>
                    <p data-testid="preview-expected-duration">
                        {format!(
                            "Expected Match Duration: {} minutes",
                            generic_config.expected_match_duration_minutes.as_secs() / 60,
                        )}
                    </p>
                </div>
            </div>
        }
        .into_any()
    }
    fn render_dropdown(&self, config: &SportConfig) -> AnyView {
        view! {
            <div class="p-2" data-testid="sport-config-dropdown">
                <span class="font-medium">{config.name.clone()}</span>
            </div>
        }
        .into_any()
    }
    fn render_configuration(&self) -> AnyView {
        view! { <GenericSportConfigForm /> }.into_any()
    }
}

#[component]
pub fn GenericSportConfigForm() -> impl IntoView {
    // --- Hooks, Navigation & global state ---
    let UseQueryNavigationReturn {
        update,
        path,
        query_string,
        ..
    } = use_query_navigation();

    let is_new = move || path.read().ends_with("/new_pa") || path.read().is_empty();
    let sport_query = use_query::<SportParams>();
    let id = Signal::derive(move || {
        if is_new() {
            None
        } else {
            sport_query.get().map(|ap| ap.sport_id).unwrap_or(None)
        }
    });
}
