//! Implementation of sport preview for the generic sport plugin

use crate::config::GenericSportConfig;

use super::GenericSportPlugin;
use app_core::{SportConfig, utils::validation::ValidationErrors};
use app_utils::components::inputs::{
    ValidatedDurationInput, ValidatedNumberInput, ValidatedOptionNumberInput,
};
use leptos::prelude::*;
use shared::{RenderCfgProps, SportConfigWebUi};
use std::time::Duration;

impl SportConfigWebUi for GenericSportPlugin {
    fn render_preview(&self, config: &SportConfig) -> AnyView {
        let generic_config = match self.validate_config(config, ValidationErrors::new()) {
            Ok(cfg) => cfg,
            Err(_) => return view! { <div>{"Invalid Configuration"}</div> }.into_any(),
        };
        view! {
            <div class="card w-full bg-base-200 shadow-md mt-4" data-testid="sport-config-preview">
                <div class="card-body">
                    <h3 class="card-title" data-testid="preview-sport-name">
                        {config.get_name().to_string()}
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
                <span class="font-medium">{config.get_name().to_string()}</span>
            </div>
        }
        .into_any()
    }
    fn render_configuration(&self, props: RenderCfgProps) -> AnyView {
        let RenderCfgProps {
            config,
            is_valid_json,
            is_new,
            is_loading,
        } = props;

        // --- extract current configuration ---
        let current_configuration = move || {
            if let Some(json_cfg) = config.get()
                && let Ok(cfg) = GenericSportConfig::parse_config(json_cfg)
            {
                cfg
            } else {
                GenericSportConfig::default()
            }
        };

        let validation_result = move || current_configuration().validate(ValidationErrors::new());

        Effect::new(move || {
            is_valid_json.set(validation_result().is_ok());
        });

        // --- Simplified Validation Closures ---
        let is_field_valid = move |field: &str| match validation_result() {
            Ok(_) => true,
            Err(err) => err.errors.iter().all(|e| e.get_field() != field),
        };

        let is_valid_sets_to_win = Signal::derive(move || is_field_valid("sets_to_win"));
        let is_valid_score_to_win = Signal::derive(move || is_field_valid("score_to_win"));
        let is_valid_win_by_margin = Signal::derive(move || is_field_valid("win_by_margin"));
        let is_valid_hard_cap = Signal::derive(move || is_field_valid("hard_cap"));
        let is_valid_victory_points_win =
            Signal::derive(move || is_field_valid("victory_points_win"));
        let is_valid_victory_points_draw =
            Signal::derive(move || is_field_valid("victory_points_draw"));
        let is_valid_expected_match_duration_minutes =
            Signal::derive(move || is_field_valid("expected_match_duration_minutes"));

        // --- Signals for form fields ---
        let set_sets_to_win = RwSignal::new(0_u16);
        let set_score_to_win = RwSignal::new(None::<u16>);
        let set_win_by_margin = RwSignal::new(None::<u16>);
        let set_hard_cap = RwSignal::new(None::<u16>);
        let set_victory_points_win = RwSignal::new(0.0_f32);
        let set_victory_points_draw = RwSignal::new(0.0_f32);
        let set_expected_match_duration_minutes = RwSignal::new(Duration::from_secs(0));

        Effect::new(move || {
            let cfg = current_configuration();
            set_sets_to_win.set(cfg.sets_to_win);
            set_score_to_win.set(cfg.score_to_win);
            set_win_by_margin.set(cfg.win_by_margin);
            set_hard_cap.set(cfg.hard_cap);
            set_victory_points_win.set(cfg.victory_points_win);
            set_victory_points_draw.set(cfg.victory_points_draw);
            set_expected_match_duration_minutes.set(cfg.expected_match_duration_minutes);
        });

        view! {
            <div class="space-y-4" data-testid="sport-config-configuration">
                <ValidatedNumberInput<
                u16,
            >
                    label="Sets to Win"
                    name="sets_to_win"
                    value=set_sets_to_win
                    is_valid=is_valid_sets_to_win
                    is_loading=is_loading
                    is_new=is_new
                    min="1"
                    on_blur=move || {
                        let mut cfg = current_configuration();
                        cfg.sets_to_win = set_sets_to_win.get();
                        config.set(serde_json::to_value(cfg).ok());
                    }
                />
                <ValidatedOptionNumberInput<
                u16,
            >
                    label="Score to Win a Set"
                    name="score_to_win"
                    value=set_score_to_win
                    is_valid=is_valid_score_to_win
                    is_loading=is_loading
                    is_new=is_new
                    min="1"
                    on_blur=move || {
                        let mut cfg = current_configuration();
                        cfg.score_to_win = set_score_to_win.get();
                        config.set(serde_json::to_value(cfg).ok());
                    }
                />
                <ValidatedOptionNumberInput<
                u16,
            >
                    label="Win by Margin"
                    name="win_by_margin"
                    value=set_win_by_margin
                    is_valid=is_valid_win_by_margin
                    is_loading=is_loading
                    is_new=is_new
                    min="1"
                    on_blur=move || {
                        let mut cfg = current_configuration();
                        cfg.win_by_margin = set_win_by_margin.get();
                        config.set(serde_json::to_value(cfg).ok());
                    }
                />
                <ValidatedOptionNumberInput<
                u16,
            >
                    label="Hard Cap"
                    name="hard_cap"
                    value=set_hard_cap
                    is_valid=is_valid_hard_cap
                    is_loading=is_loading
                    is_new=is_new
                    min="1"
                    on_blur=move || {
                        let mut cfg = current_configuration();
                        cfg.hard_cap = set_hard_cap.get();
                        config.set(serde_json::to_value(cfg).ok());
                    }
                />
                <ValidatedNumberInput<
                f32,
            >
                    label="Victory Points for Win"
                    name="victory_points_win"
                    value=set_victory_points_win
                    is_valid=is_valid_victory_points_win
                    is_loading=is_loading
                    is_new=is_new
                    min="0"
                    step="0.1"
                    on_blur=move || {
                        let mut cfg = current_configuration();
                        cfg.victory_points_win = set_victory_points_win.get();
                        config.set(serde_json::to_value(cfg).ok());
                    }
                />
                <ValidatedNumberInput<
                f32,
            >
                    label="Victory Points for Draw"
                    name="victory_points_draw"
                    value=set_victory_points_draw
                    is_valid=is_valid_victory_points_draw
                    is_loading=is_loading
                    is_new=is_new
                    min="0"
                    step="0.1"
                    on_blur=move || {
                        let mut cfg = current_configuration();
                        cfg.victory_points_draw = set_victory_points_draw.get();
                        config.set(serde_json::to_value(cfg).ok());
                    }
                />
                <ValidatedDurationInput
                    label="Expected Match Duration (minutes)"
                    name="expected_match_duration_minutes"
                    value=set_expected_match_duration_minutes
                    is_valid=is_valid_expected_match_duration_minutes
                    is_loading=is_loading
                    is_new=is_new
                    on_blur=move || {
                        let mut cfg = current_configuration();
                        cfg.expected_match_duration_minutes = set_expected_match_duration_minutes
                            .get();
                        config.set(serde_json::to_value(cfg).ok());
                    }
                />
            </div>
        }
        .into_any()
    }
}
