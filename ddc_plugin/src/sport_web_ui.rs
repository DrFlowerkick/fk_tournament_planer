//! Implementation of sport preview for the generic sport plugin

use super::DdcSportPlugin;
use crate::config::{DdcSetCfg, DdcSetWinningCfg, DdcSportConfig};
use app_core::{
    SportConfig, SportPort,
    utils::validation::{ValidationErrors, ValidationResult},
};
use app_utils::{
    components::inputs::{
        DurationInputUnit, DurationInputWithValidation, EnumSelectWithValidation,
        NumberInputWithValidation,
    },
    state::sport_config_editor::SportConfigEditorContext,
};
use leptos::prelude::*;
use shared::SportPortWebUi;
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
    fn render_configuration(&self) -> AnyView {
        // get editor context
        let sport_config_editor = expect_context::<SportConfigEditorContext>();

        // --- extract current configuration ---
        let current_config = Signal::derive(move || {
            if let Some(json_cfg) = sport_config_editor.config.get()
                && let Ok(cfg) = DdcSportConfig::parse_config(json_cfg)
            {
                Some(cfg)
            } else {
                None
            }
        });

        let validation_result = Signal::derive(move || {
            if let Some(object_id) = sport_config_editor.sport_config_id.get()
                && let Some(cfg) = current_config.get()
            {
                cfg.validate(object_id, ValidationErrors::new())
            } else {
                ValidationResult::Ok(())
            }
        });

        Effect::new(move || {
            sport_config_editor
                .set_is_valid_json
                .set(validation_result.with(|vr| vr.is_ok()));
        });

        // --- Signals for form fields ---
        // configuration of sets
        let (num_sets, set_num_sets) = signal(None::<u16>);
        let set_num_sets = Callback::new(move |num_sets: Option<u16>| {
            set_num_sets.set(num_sets);
        });
        let sets_cfg =
            Signal::derive(move || current_config.with(|cfg| cfg.as_ref().map(|c| c.sets_cfg)));
        let set_sets_cfg = Callback::new(move |new_cfg: Option<DdcSetCfg>| {
            if let Some(mut cfg) = current_config.get()
                && let Some(new_cfg) = new_cfg
            {
                let new_cfg = match new_cfg {
                    DdcSetCfg::CustomSetsToWin { .. } => DdcSetCfg::CustomSetsToWin {
                        sets_to_win: num_sets.get().unwrap_or_default(),
                    },
                    DdcSetCfg::CustomTotalSets { .. } => DdcSetCfg::CustomTotalSets {
                        total_sets: num_sets.get().unwrap_or_default(),
                    },
                    _ => {
                        // reset num_sets if not custom
                        set_num_sets.run(None);
                        new_cfg
                    }
                };
                cfg.sets_cfg = new_cfg;
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        // configuration of winning a set
        let (score_to_win, set_score_to_win) = signal(None::<u16>);
        let set_score_to_win = Callback::new(move |score: Option<u16>| {
            set_score_to_win.set(score);
        });
        let (win_by_margin, set_win_by_margin) = signal(None::<u16>);
        let set_win_by_margin = Callback::new(move |margin: Option<u16>| {
            set_win_by_margin.set(margin);
        });
        let (hard_cap, set_hard_cap) = signal(None::<u16>);
        let set_hard_cap = Callback::new(move |cap: Option<u16>| {
            set_hard_cap.set(cap);
        });
        let winning_cfg = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().map(|c| c.set_winning_cfg))
        });
        let set_winning_cfg = Callback::new(move |new_cfg: Option<DdcSetWinningCfg>| {
            if let Some(mut cfg) = current_config.get()
                && let Some(new_cfg) = new_cfg
            {
                let new_cfg = match new_cfg {
                    DdcSetWinningCfg::Custom { .. } => DdcSetWinningCfg::Custom {
                        score_to_win: score_to_win.get().unwrap_or_default(),
                        win_by_margin: win_by_margin.get().unwrap_or_default(),
                        hard_cap: hard_cap.get().unwrap_or_default(),
                    },
                    _ => {
                        // reset fields if not custom
                        set_score_to_win.run(None);
                        set_win_by_margin.run(None);
                        set_hard_cap.run(None);
                        new_cfg
                    }
                };
                cfg.set_winning_cfg = new_cfg;
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        let victory_points_win = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().map(|c| c.victory_points_win))
        });
        let set_victory_points_win = Callback::new(move |points: Option<f32>| {
            if let Some(mut cfg) = current_config.get() {
                cfg.victory_points_win = points.unwrap_or_default();
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        let victory_points_draw = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().map(|c| c.victory_points_draw))
        });
        let set_victory_points_draw = Callback::new(move |points: Option<f32>| {
            if let Some(mut cfg) = current_config.get() {
                cfg.victory_points_draw = points.unwrap_or_default();
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        let expected_rally_duration_seconds = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().map(|c| c.expected_rally_duration_seconds))
        });
        let set_expected_rally_duration_seconds = Callback::new(move |duration: Duration| {
            if let Some(mut cfg) = current_config.get() {
                cfg.expected_rally_duration_seconds = duration;
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });

        view! {
            <div class="space-y-4" data-testid="sport-config-configuration">
                <EnumSelectWithValidation
                    label="Sets Configuration"
                    name="sets_cfg"
                    value=sets_cfg
                    set_value=set_sets_cfg
                />
                {move || {
                    match sets_cfg.get() {
                        Some(DdcSetCfg::CustomSetsToWin { .. }) => {
                            view! {
                                <NumberInputWithValidation
                                    label="Sets to Win"
                                    name="num_sets"
                                    value=num_sets
                                    set_value=set_num_sets
                                    validation_result=validation_result
                                    object_id=sport_config_editor.sport_config_id
                                    field="sets_cfg"
                                    min="1"
                                />
                            }
                                .into_any()
                        }
                        Some(DdcSetCfg::CustomTotalSets { .. }) => {
                            view! {
                                <NumberInputWithValidation
                                    label="Total Sets"
                                    name="num_sets"
                                    value=num_sets
                                    set_value=set_num_sets
                                    validation_result=validation_result
                                    object_id=sport_config_editor.sport_config_id
                                    field="sets_cfg"
                                    min="1"
                                />
                            }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }
                }}
                <EnumSelectWithValidation
                    label="Set Winning Configuration"
                    name="set_winning_cfg"
                    value=winning_cfg
                    set_value=set_winning_cfg
                />
                {move || {
                    match winning_cfg.get() {
                        Some(DdcSetWinningCfg::Custom { .. }) => {
                            view! {
                                <div class="grid grid-cols-3 gap-4">
                                    <NumberInputWithValidation
                                        label="Score to Win a Set"
                                        name="score_to_win"
                                        value=score_to_win
                                        set_value=set_score_to_win
                                        validation_result=validation_result
                                        object_id=sport_config_editor.sport_config_id
                                        field="score_to_win"
                                        min="1"
                                    />
                                    <NumberInputWithValidation
                                        label="Win by Margin"
                                        name="win_by_margin"
                                        value=win_by_margin
                                        set_value=set_win_by_margin
                                        validation_result=validation_result
                                        object_id=sport_config_editor.sport_config_id
                                        field="win_by_margin"
                                        min="1"
                                    />
                                    <NumberInputWithValidation
                                        label="Hard Cap"
                                        name="hard_cap"
                                        value=hard_cap
                                        set_value=set_hard_cap
                                        validation_result=validation_result
                                        object_id=sport_config_editor.sport_config_id
                                        field="hard_cap"
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
                    <NumberInputWithValidation
                        label="Victory Points for Win"
                        name="victory_points_win"
                        value=victory_points_win
                        set_value=set_victory_points_win
                        validation_result=validation_result
                        object_id=sport_config_editor.sport_config_id
                        field="victory_points_win"
                        min="0"
                        step="0.1"
                    />
                    <NumberInputWithValidation
                        label="Victory Points for Draw"
                        name="victory_points_draw"
                        value=victory_points_draw
                        set_value=set_victory_points_draw
                        validation_result=validation_result
                        object_id=sport_config_editor.sport_config_id
                        field="victory_points_draw"
                        min="0"
                        step="0.1"
                    />
                </div>
                <DurationInputWithValidation
                    label="Expected Rally Duration"
                    name="expected_rally_duration_seconds"
                    value=expected_rally_duration_seconds
                    set_value=set_expected_rally_duration_seconds
                    unit=DurationInputUnit::Seconds
                    validation_result=validation_result
                    object_id=sport_config_editor.sport_config_id
                    field="expected_rally_duration_seconds"
                />
                <div class="form-control w-full">
                    <label class="label">
                        <span class="label-text">"Estimated Match Duration"</span>
                    </label>
                    <div class="input input-bordered flex items-center bg-base-200 text-base-content/70 cursor-not-allowed">
                        {move || {
                            let minutes = current_config
                                .with(|cfg_opt| {
                                    if let Some(cfg) = cfg_opt {
                                        cfg.estimate_match_duration().as_secs() / 60
                                    } else {
                                        0
                                    }
                                });
                            format!("{minutes} minutes")
                        }}
                    </div>
                </div>
            </div>
        }
        .into_any()
    }
}
