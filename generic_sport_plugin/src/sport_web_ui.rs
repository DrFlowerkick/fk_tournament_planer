//! Implementation of sport preview for the generic sport plugin

use crate::config::GenericSportConfig;

use super::GenericSportPlugin;
use app_core::{SportConfig, utils::validation::ValidationErrors};
use app_utils::{
    components::inputs::{
        DurationInputUnit, ValidatedDurationInput, ValidatedNumberInput, ValidatedOptionNumberInput,
    },
    hooks::is_field_valid::is_field_valid,
};
use leptos::prelude::*;
use shared::{RenderCfgProps, SportPortWebUi};
use std::time::Duration;

impl SportPortWebUi for GenericSportPlugin {
    fn render_plugin_selection(&self) -> AnyView {
        view! {
            <div
                class="flex flex-col items-center justify-center gap-4 w-full"
                data-testid="generic-sport-plugin-selection"
            >
                // Placeholder icon/emoji
                <div class="text-6xl">"🏆"</div>
                <div class="font-bold text-xl uppercase tracking-wider text-center">
                    "Generic Sport"
                </div>
            </div>
        }
        .into_any()
    }
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
            object_id,
            config,
            is_valid_json,
        } = props;

        // --- extract current configuration ---
        // ToDo: refactor this to keep the Result of parse in a Signal and use below ErrorBoundary to investigate the error.
        // For now, we ignore errors in parsing here, as validation is done below.
        let current_configuration = move || {
            if let Some(json_cfg) = config.get()
                && let Ok(cfg) = GenericSportConfig::parse_config(json_cfg)
            {
                cfg
            } else {
                GenericSportConfig::default()
            }
        };

        let validation_result = Signal::derive(move || {
            current_configuration().validate(object_id.get(), ValidationErrors::new())
        });

        Effect::new(move || {
            is_valid_json.set(validation_result.get().is_ok());
        });

        let is_valid_sets_to_win =
            Signal::derive(move || is_field_valid(validation_result, "sets_to_win"));
        let is_valid_score_to_win =
            Signal::derive(move || is_field_valid(validation_result, "score_to_win"));
        let is_valid_win_by_margin =
            Signal::derive(move || is_field_valid(validation_result, "win_by_margin"));
        let is_valid_hard_cap =
            Signal::derive(move || is_field_valid(validation_result, "hard_cap"));
        let is_valid_victory_points_win =
            Signal::derive(move || is_field_valid(validation_result, "victory_points_win"));
        let is_valid_victory_points_draw =
            Signal::derive(move || is_field_valid(validation_result, "victory_points_draw"));
        let is_valid_expected_match_duration_minutes = Signal::derive(move || {
            is_field_valid(validation_result, "expected_match_duration_minutes")
        });
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

        // Synchronize changes from local signals back to the main config
        // accessible via 'config'. Using tracking signals ensures updates happen
        // regardless of focus/blur events (fixing Firefox spinner issue).
        Effect::new(move || {
            let sets = set_sets_to_win.get();
            let score = set_score_to_win.get();
            let margin = set_win_by_margin.get();
            let cap = set_hard_cap.get();
            let vp_win = set_victory_points_win.get();
            let vp_draw = set_victory_points_draw.get();
            let duration = set_expected_match_duration_minutes.get();

            // Use untrack to prevent cyclic dependencies. We only want to push
            // *up* to config when local signals change, not re-run when config changes.
            untrack(move || {
                let mut cfg = current_configuration();
                let mut changed = false;

                if cfg.sets_to_win != sets {
                    cfg.sets_to_win = sets;
                    changed = true;
                }
                if cfg.score_to_win != score {
                    cfg.score_to_win = score;
                    changed = true;
                }
                if cfg.win_by_margin != margin {
                    cfg.win_by_margin = margin;
                    changed = true;
                }
                if cfg.hard_cap != cap {
                    cfg.hard_cap = cap;
                    changed = true;
                }
                if (cfg.victory_points_win - vp_win).abs() > f32::EPSILON {
                    cfg.victory_points_win = vp_win;
                    changed = true;
                }
                if (cfg.victory_points_draw - vp_draw).abs() > f32::EPSILON {
                    cfg.victory_points_draw = vp_draw;
                    changed = true;
                }
                if cfg.expected_match_duration_minutes != duration {
                    cfg.expected_match_duration_minutes = duration;
                    changed = true;
                }

                if changed {
                    config.set(serde_json::to_value(cfg).ok());
                }
            });
        });

        view! {
            <div class="space-y-4" data-testid="sport-config-configuration">
                <ValidatedNumberInput<
                u16,
            >
                    label="Sets to Win"
                    name="sets_to_win"
                    value=set_sets_to_win
                    validation_error=is_valid_sets_to_win
                    min="1"
                />
                <div class="grid grid-cols-3 gap-4">
                    <ValidatedOptionNumberInput<
                    u16,
                >
                        label="Score to Win a Set"
                        name="score_to_win"
                        value=set_score_to_win
                        validation_error=is_valid_score_to_win
                        min="1"
                    />
                    <ValidatedOptionNumberInput<
                    u16,
                >
                        label="Win by Margin"
                        name="win_by_margin"
                        value=set_win_by_margin
                        validation_error=is_valid_win_by_margin
                        min="1"
                    />
                    <ValidatedOptionNumberInput<
                    u16,
                >
                        label="Hard Cap"
                        name="hard_cap"
                        value=set_hard_cap
                        validation_error=is_valid_hard_cap
                        min="1"
                    />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <ValidatedNumberInput<
                    f32,
                >
                        label="Victory Points for Win"
                        name="victory_points_win"
                        value=set_victory_points_win
                        validation_error=is_valid_victory_points_win
                        min="0"
                        step="0.1"
                    />
                    <ValidatedNumberInput<
                    f32,
                >
                        label="Victory Points for Draw"
                        name="victory_points_draw"
                        value=set_victory_points_draw
                        validation_error=is_valid_victory_points_draw
                        min="0"
                        step="0.1"
                    />
                </div>
                <ValidatedDurationInput
                    label="Expected Match Duration"
                    name="expected_match_duration_minutes"
                    value=set_expected_match_duration_minutes
                    unit=DurationInputUnit::Minutes
                    validation_error=is_valid_expected_match_duration_minutes
                />
            </div>
        }
        .into_any()
    }
}
