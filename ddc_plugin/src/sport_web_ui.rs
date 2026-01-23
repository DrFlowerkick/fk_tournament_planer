//! Implementation of sport preview for the generic sport plugin

use crate::config::{DdcSetCfg, DdcSetWinningCfg, DdcSportConfig};

use super::DdcSportPlugin;
use app_core::{SportConfig, SportPort, utils::validation::ValidationErrors};
use app_utils::{
    components::inputs::{
        DurationInputUnit, EnumSelect, ValidatedDurationInput, ValidatedNumberInput,
    },
    hooks::is_field_valid::is_field_valid,
};
use leptos::prelude::*;
use shared::{RenderCfgProps, SportPortWebUi};
use std::time::Duration;

impl SportPortWebUi for DdcSportPlugin {
    fn render_plugin_selection(&self) -> AnyView {
        view! {
            <div
                class="flex flex-col items-center justify-center gap-4 w-full"
                data-testid="ddc-plugin-selection"
            >
                // DDC Icon / Emoji
                // ToDo: Replace with proper icon later
                <div class="text-6xl">"🥏"</div>
                <div class="flex flex-col items-center">
                    <h3 class="text-xl font-bold text-center">"Double Disc Court"</h3>
                    <span class="text-xs uppercase tracking-widest opacity-70">"DDC"</span>
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
                    <p data-testid="preview-set-config">{generic_config.sets_cfg.to_string()}</p>
                    <p data-testid="preview-set-winning-config">
                        {generic_config.set_winning_cfg.to_string()}
                    </p>
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
                            Self::new()
                                .estimate_match_duration(config)
                                .unwrap_or(Duration::from_secs(0))
                                .as_secs() / 60,
                        )}
                    </p>
                </div>
            </div>
        }
        .into_any()
    }
    fn render_dropdown(&self, config: &SportConfig) -> AnyView {
        let generic_config = match self.validate_config(config, ValidationErrors::new()) {
            Ok(cfg) => cfg,
            Err(_) => return view! { <div>{"Invalid Configuration"}</div> }.into_any(),
        };
        view! {
            <div class="p-2" data-testid="sport-config-dropdown">
                <span class="font-medium">{config.get_name().to_string()}</span>
                <span class="font-medium">{generic_config.sets_cfg.to_string()}</span>
                <span class="font-medium">{generic_config.set_winning_cfg.to_string()}</span>
            </div>
        }
        .into_any()
    }
    fn render_configuration(&self, props: RenderCfgProps) -> AnyView {
        let RenderCfgProps {
            config,
            is_valid_json,
        } = props;

        // --- extract current configuration ---
        // ToDo: refactor this to keep the Result of parse in a Signal and use below ErrorBoundary to investigate the error.
        // For now, we ignore errors in parsing here, as validation is done below.
        let current_configuration = move || {
            if let Some(json_cfg) = config.get()
                && let Ok(cfg) = DdcSportConfig::parse_config(json_cfg)
            {
                cfg
            } else {
                DdcSportConfig::default()
            }
        };

        let validation_result =
            Signal::derive(move || current_configuration().validate(ValidationErrors::new()));

        Effect::new(move || {
            is_valid_json.set(validation_result.get().is_ok());
        });

        let is_valid_sets_cfg =
            Signal::derive(move || is_field_valid(validation_result, "sets_cfg"));
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
        let is_valid_expected_rally_duration_seconds = Signal::derive(move || {
            is_field_valid(validation_result, "expected_rally_duration_seconds")
        });
        // --- Signals for form fields ---
        let set_sets_cfg = RwSignal::new(DdcSetCfg::default());
        let set_num_sets = RwSignal::new(0_u16);
        let set_winning_cfg = RwSignal::new(DdcSetWinningCfg::default());
        let set_score_to_win = RwSignal::new(0_u16);
        let set_win_by_margin = RwSignal::new(0_u16);
        let set_hard_cap = RwSignal::new(0_u16);
        let set_victory_points_win = RwSignal::new(0.0_f32);
        let set_victory_points_draw = RwSignal::new(0.0_f32);
        let set_expected_rally_duration_seconds = RwSignal::new(Duration::from_secs(0));

        Effect::new(move || {
            let cfg = current_configuration();
            set_sets_cfg.set(cfg.sets_cfg);
            set_num_sets.set(cfg.sets_cfg.sets_to_play().0);
            set_winning_cfg.set(cfg.set_winning_cfg);
            let (score_to_win, win_by_margin, hard_cap) = cfg.set_winning_cfg.get_win_cfg();
            set_score_to_win.set(score_to_win);
            set_win_by_margin.set(win_by_margin);
            set_hard_cap.set(hard_cap);
            set_victory_points_win.set(cfg.victory_points_win);
            set_victory_points_draw.set(cfg.victory_points_draw);
            set_expected_rally_duration_seconds.set(cfg.expected_rally_duration_seconds);
        });

        // Sync local inputs back to JSON config continuously using an Effect
        Effect::new(move || {
            let sets_cfg_enum = set_sets_cfg.get();
            let num_sets = set_num_sets.get();
            let winning_cfg_enum = set_winning_cfg.get();
            let score = set_score_to_win.get();
            let margin = set_win_by_margin.get();
            let cap = set_hard_cap.get();
            let vp_win = set_victory_points_win.get();
            let vp_draw = set_victory_points_draw.get();
            let rally_dur = set_expected_rally_duration_seconds.get();

            untrack(move || {
                let mut cfg = current_configuration();
                let mut changed = false;

                // 1. Reconstruct DdcSetCfg
                // We merge the enum variant (from dropdown) with the number value (from input)
                let new_sets_cfg = match sets_cfg_enum {
                    DdcSetCfg::CustomSetsToWin { .. } => DdcSetCfg::CustomSetsToWin {
                        sets_to_win: num_sets,
                    },
                    DdcSetCfg::CustomTotalSets { .. } => DdcSetCfg::CustomTotalSets {
                        total_sets: num_sets,
                    },
                    // For standard variants, the num_sets signal is derived but not authoritative for the config structure
                    _ => sets_cfg_enum.clone(),
                };

                if cfg.sets_cfg != new_sets_cfg {
                    cfg.sets_cfg = new_sets_cfg;
                    changed = true;
                }

                // 2. Reconstruct DdcSetWinningCfg
                let new_winning_cfg = match winning_cfg_enum {
                    DdcSetWinningCfg::Custom { .. } => DdcSetWinningCfg::Custom {
                        score_to_win: score,
                        win_by_margin: margin,
                        hard_cap: cap,
                    },
                    _ => winning_cfg_enum.clone(),
                };

                if cfg.set_winning_cfg != new_winning_cfg {
                    cfg.set_winning_cfg = new_winning_cfg;
                    changed = true;
                }

                // 3. Simple fields
                if (cfg.victory_points_win - vp_win).abs() > f32::EPSILON {
                    cfg.victory_points_win = vp_win;
                    changed = true;
                }
                if (cfg.victory_points_draw - vp_draw).abs() > f32::EPSILON {
                    cfg.victory_points_draw = vp_draw;
                    changed = true;
                }
                if cfg.expected_rally_duration_seconds != rally_dur {
                    cfg.expected_rally_duration_seconds = rally_dur;
                    changed = true;
                }

                if changed {
                    config.set(serde_json::to_value(cfg).ok());
                }
            });
        });

        view! {
            <div class="space-y-4" data-testid="sport-config-configuration">
                <EnumSelect
                    label="Sets Configuration"
                    name="sets_cfg"
                    value=set_sets_cfg
                    on_change=move || {
                        set_sets_cfg
                            .update(|v| match v {
                                DdcSetCfg::CustomSetsToWin { sets_to_win } => {
                                    *sets_to_win = set_num_sets.get();
                                }
                                DdcSetCfg::CustomTotalSets { total_sets } => {
                                    *total_sets = set_num_sets.get();
                                }
                                _ => {
                                    set_num_sets.set(v.sets_to_play().0);
                                }
                            });
                    }
                />
                {move || {
                    match set_sets_cfg.get() {
                        DdcSetCfg::CustomSetsToWin { .. } => {
                            view! {
                                <ValidatedNumberInput<
                                u16,
                            >
                                    label="Sets to Win"
                                    name="num_sets"
                                    value=set_num_sets
                                    validation_error=is_valid_sets_cfg
                                    min="1"
                                />
                            }
                                .into_any()
                        }
                        DdcSetCfg::CustomTotalSets { .. } => {
                            view! {
                                <ValidatedNumberInput<
                                u16,
                            >
                                    label="Total Sets"
                                    name="num_sets"
                                    value=set_num_sets
                                    validation_error=is_valid_sets_cfg
                                    min="1"
                                />
                            }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }
                }}
                <EnumSelect
                    label="Set Winning Configuration"
                    name="set_winning_cfg"
                    value=set_winning_cfg
                    on_change=move || {
                        set_winning_cfg
                            .update(|v| match v {
                                DdcSetWinningCfg::Custom {
                                    score_to_win,
                                    win_by_margin,
                                    hard_cap,
                                } => {
                                    *score_to_win = set_score_to_win.get();
                                    *win_by_margin = set_win_by_margin.get();
                                    *hard_cap = set_hard_cap.get();
                                }
                                _ => {
                                    let (score_to_win, win_by_margin, hard_cap) = v.get_win_cfg();
                                    set_score_to_win.set(score_to_win);
                                    set_win_by_margin.set(win_by_margin);
                                    set_hard_cap.set(hard_cap);
                                }
                            });
                    }
                />
                {move || {
                    match set_winning_cfg.get() {
                        DdcSetWinningCfg::Custom { .. } => {
                            view! {
                                <div class="grid grid-cols-3 gap-4">
                                    <ValidatedNumberInput<
                                    u16,
                                >
                                        label="Score to Win a Set"
                                        name="score_to_win"
                                        value=set_score_to_win
                                        validation_error=is_valid_score_to_win
                                        min="1"
                                    />
                                    <ValidatedNumberInput<
                                    u16,
                                >
                                        label="Win by Margin"
                                        name="win_by_margin"
                                        value=set_win_by_margin
                                        validation_error=is_valid_win_by_margin
                                        min="1"
                                    />
                                    <ValidatedNumberInput<
                                    u16,
                                >
                                        label="Hard Cap"
                                        name="hard_cap"
                                        value=set_hard_cap
                                        validation_error=is_valid_hard_cap
                                        min="1"
                                    />
                                </div>
                            }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }
                }}
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
                    label="Expected Rally Duration"
                    name="expected_rally_duration_seconds"
                    value=set_expected_rally_duration_seconds
                    unit=DurationInputUnit::Seconds
                    validation_error=is_valid_expected_rally_duration_seconds
                />
                <div class="form-control w-full">
                    <label class="label">
                        <span class="label-text">"Estimated Match Duration"</span>
                    </label>
                    <div class="input input-bordered flex items-center bg-base-200 text-base-content/70 cursor-not-allowed">
                        {move || {
                            let cfg = current_configuration();
                            format!("{} minutes", cfg.estimate_match_duration().as_secs() / 60)
                        }}
                    </div>
                </div>
            </div>
        }
        .into_any()
    }
}
