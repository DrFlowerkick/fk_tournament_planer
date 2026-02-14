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
    state::sport_config::SportConfigEditorContext,
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
    fn render_detailed_preview(&self, config: &SportConfig) -> AnyView {
        let generic_config = match self.validate_config(config, ValidationErrors::new()) {
            Ok(cfg) => cfg,
            Err(_) => return view! { <div>{"Invalid Configuration"}</div> }.into_any(),
        };

        let duration_minutes = Self::new()
            .estimate_match_duration(config)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
            / 60;

        view! {
            <div
                class="flex flex-wrap items-center gap-x-4 gap-y-2 p-3 bg-base-200 text-sm rounded-lg"
                data-testid="table-entry-detailed-preview"
            >
                // 1. Block: Match Rules (Sets & Scoring)
                <div class="flex flex-wrap items-center gap-2">
                    <div class="flex items-center gap-1 font-semibold text-base-content">
                        <span class="icon-[heroicons--trophy] w-4 h-4 opacity-70"></span>
                        <span data-testid="preview-set-config">
                            {generic_config.sets_cfg.to_string()}
                        </span>
                    </div>

                    <span class="hidden sm:inline text-base-content/30">"|"</span>

                    <span class="text-base-content/80" data-testid="preview-set-winning-config">
                        {generic_config.set_winning_cfg.to_string()}
                    </span>
                </div>

                // Spacer to push meta info to the right on larger screens if desired,
                // or just wrap naturally. Here we keep them close.

                // 2. Block: Meta Info (Duration & Points)
                <div class="flex items-center gap-3 ml-auto sm:ml-0">
                    // Duration
                    <div
                        class="flex items-center gap-1 text-xs opacity-70"
                        title="Expected Match Duration"
                    >
                        <span class="icon-[heroicons--clock] w-4 h-4"></span>
                        <span data-testid="preview-expected-duration">
                            {format!("~{} min", duration_minutes)}
                        </span>
                    </div>

                    // Victory Points Badges
                    <div class="flex gap-1">
                        <span
                            class="badge badge-sm badge-success badge-outline gap-1"
                            title="Victory Points (Win)"
                            data-testid="preview-victory-points-win"
                        >
                            <span class="font-bold">"W"</span>
                            {generic_config.victory_points_win}
                        </span>
                        <span
                            class="badge badge-sm badge-ghost badge-outline gap-1"
                            title="Victory Points (Draw)"
                            data-testid="preview-victory-points-draw"
                        >
                            <span class="font-bold">"D"</span>
                            {generic_config.victory_points_draw}
                        </span>
                    </div>
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
            <div class="p-2">
                <span class="font-medium">{generic_config.sets_cfg.to_string()}</span>
                <span class="hidden sm:inline text-base-content/30">"|"</span>
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
        let num_sets = Signal::derive(move || {
            current_config.with(|cfg| {
                if let Some(cfg) = cfg {
                    match cfg.sets_cfg {
                        DdcSetCfg::CustomSetsToWin { sets_to_win } => Some(sets_to_win),
                        DdcSetCfg::CustomTotalSets { total_sets } => Some(total_sets),
                        _ => None,
                    }
                } else {
                    None
                }
            })
        });
        let sets_cfg =
            Signal::derive(move || current_config.with(|cfg| cfg.as_ref().map(|c| c.sets_cfg)));
        let set_sets_cfg = Callback::new(move |new_cfg: Option<DdcSetCfg>| {
            if let Some(mut cfg) = current_config.get()
                && let Some(new_cfg) = new_cfg
            {
                cfg.sets_cfg = new_cfg;
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        let set_num_sets = Callback::new(move |num_sets: Option<u16>| {
            if let Some(cfg) = current_config.get()
                && let Some(num_sets) = num_sets
            {
                match cfg.sets_cfg {
                    DdcSetCfg::CustomSetsToWin { .. } => {
                        set_sets_cfg.run(Some(DdcSetCfg::CustomSetsToWin {
                            sets_to_win: num_sets,
                        }));
                    }
                    DdcSetCfg::CustomTotalSets { .. } => {
                        set_sets_cfg.run(Some(DdcSetCfg::CustomTotalSets {
                            total_sets: num_sets,
                        }));
                    }
                    _ => {}
                }
            }
        });
        // configuration of winning a set
        let score_to_win = Signal::derive(move || {
            current_config.with(|cfg| {
                if let Some(cfg) = cfg {
                    match cfg.set_winning_cfg {
                        DdcSetWinningCfg::Custom { score_to_win, .. } => Some(score_to_win),
                        _ => None,
                    }
                } else {
                    None
                }
            })
        });
        let set_score_to_win = Callback::new(move |score: Option<u16>| {
            if let Some(mut cfg) = current_config.get() {
                if let Some(score) = score {
                    let mut changed = false;
                    match &mut cfg.set_winning_cfg {
                        DdcSetWinningCfg::Custom { score_to_win, .. } => {
                            *score_to_win = score;
                            changed = true;
                        }
                        _ => {}
                    }
                    if changed {
                        sport_config_editor
                            .set_config
                            .set(serde_json::to_value(cfg).unwrap());
                    }
                }
            }
        });
        let win_by_margin = Signal::derive(move || {
            current_config.with(|cfg| {
                if let Some(cfg) = cfg {
                    match cfg.set_winning_cfg {
                        DdcSetWinningCfg::Custom { win_by_margin, .. } => Some(win_by_margin),
                        _ => None,
                    }
                } else {
                    None
                }
            })
        });
        let set_win_by_margin = Callback::new(move |margin: Option<u16>| {
            if let Some(mut cfg) = current_config.get() {
                if let Some(margin) = margin {
                    let mut changed = false;
                    match &mut cfg.set_winning_cfg {
                        DdcSetWinningCfg::Custom { win_by_margin, .. } => {
                            *win_by_margin = margin;
                            changed = true;
                        }
                        _ => {}
                    }
                    if changed {
                        sport_config_editor
                            .set_config
                            .set(serde_json::to_value(cfg).unwrap());
                    }
                }
            }
        });
        let hard_cap = Signal::derive(move || {
            current_config.with(|cfg| {
                if let Some(cfg) = cfg {
                    match cfg.set_winning_cfg {
                        DdcSetWinningCfg::Custom { hard_cap, .. } => Some(hard_cap),
                        _ => None,
                    }
                } else {
                    None
                }
            })
        });
        let set_hard_cap = Callback::new(move |cap: Option<u16>| {
            if let Some(mut cfg) = current_config.get() {
                if let Some(cap) = cap {
                    let mut changed = false;
                    match &mut cfg.set_winning_cfg {
                        DdcSetWinningCfg::Custom { hard_cap, .. } => {
                            *hard_cap = cap;
                            changed = true;
                        }
                        _ => {}
                    }
                    if changed {
                        sport_config_editor
                            .set_config
                            .set(serde_json::to_value(cfg).unwrap());
                    }
                }
            }
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
        let set_expected_rally_duration_seconds =
            Callback::new(move |duration: Option<Duration>| {
                if let Some(mut cfg) = current_config.get() {
                    cfg.expected_rally_duration_seconds =
                        duration.unwrap_or(Duration::from_secs(0));
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
                    data_testid="select-sets_cfg"
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
                                    data_testid="input-num_sets"
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
                                    data_testid="input-num_sets"
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
                    data_testid="select-set_winning_cfg"
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
                                        data_testid="input-score_to_win"
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
                                        data_testid="input-win_by_margin"
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
                                        data_testid="input-hard_cap"
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
                        data_testid="input-victory_points_win"
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
                        data_testid="input-victory_points_draw"
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
                    data_testid="input-expected_rally_duration_seconds"
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
