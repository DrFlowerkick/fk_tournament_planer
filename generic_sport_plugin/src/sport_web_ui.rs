//! Implementation of sport preview for the generic sport plugin

use std::time::Duration;

use super::GenericSportPlugin;
use app_core::SportConfig;
use app_utils::{
    global_state::{GlobalState, GlobalStateStoreFields},
    hooks::use_query_navigation::{UseQueryNavigationReturn, use_query_navigation},
    params::{SportConfigParams, SportParams},
};
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use reactive_stores::Store;
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

    let is_new = move || path.read().ends_with("/new_sc") || path.read().is_empty();
    let sport_query = use_query::<SportParams>();
    let sport_config_query = use_query::<SportConfigParams>();
    let sport_config_id = Signal::derive(move || {
        if is_new() {
            None
        } else {
            sport_config_query
                .get()
                .map(|sc| sc.sport_config_id)
                .unwrap_or(None)
        }
    });

    let state = expect_context::<Store<GlobalState>>();
    let return_after_sport_config_edit = state.return_after_sport_config_edit();
    let cancel_target =
        move || format!("{}{}", return_after_sport_config_edit.get(), query_string.get());
    
    // --- Signals for form fields ---
    let set_name = RwSignal::new(0_u16);
    let set_sets_to_win = RwSignal::new(None::<u16>);
    let set_score_to_win = RwSignal::new(None::<u16>);
    let set_win_by_margin = RwSignal::new(None::<u16>);
    let set_hard_cap = RwSignal::new(None::<u16>);
    let set_victory_points_win = RwSignal::new(0.0_f32);
    let set_victory_points_draw = RwSignal::new(0.0_f32);
    let set_expected_match_duration_minutes = RwSignal::new(Duration::from_secs(0));
    let set_version = RwSignal::new(0);

    // --- Server Resources ---
}
