//! Implementation of sport preview for the generic sport plugin

use crate::config::GenericSportConfig;

use super::GenericSportPlugin;
use app_core::{
    SportConfig,
    utils::validation::{ValidationErrors, ValidationResult},
};
use app_utils::{
    components::inputs::{
        DurationInputUnit, DurationInputWithValidation, NumberInputWithValidation,
    },
    state::sport_config::SportConfigEditorContext,
};
use leptos::prelude::*;
use shared::SportPortWebUi;
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
    fn render_detailed_preview(&self, config: &SportConfig) -> AnyView {
        let generic_config = match self.validate_config(config, ValidationErrors::new()) {
            Ok(cfg) => cfg,
            Err(_) => return view! { <div>{"Invalid Configuration"}</div> }.into_any(),
        };

        let duration_minutes = generic_config.expected_match_duration_minutes.as_secs() / 60;

        view! {
            <div
                class="flex flex-wrap items-center gap-x-4 gap-y-2 p-3 bg-base-200 text-sm rounded-lg"
                data-testid="table-entry-detailed-preview"
            >
                // 1. Block: Match Rules (Sets & Scoring)
                <div class="flex flex-wrap items-center gap-2">
                    <div class="flex items-center gap-1 font-semibold text-base-content">
                        <span class="icon-[heroicons--trophy] w-4 h-4 opacity-70"></span>
                        <span data-testid="preview-sets-to-win">
                            {format!("Sets to win: {}", generic_config.sets_to_win)}
                        </span>
                    </div>

                    <span class="hidden sm:inline text-base-content/30">"|"</span>

                    <span class="text-base-content/80" data-testid="preview-score-config">
                        {move || generic_config.display_score_limit()}
                    </span>
                </div>

                // 2. Block: Meta Info (Duration & Points)
                <div class="flex items-center gap-3 ml-auto sm:ml-0">
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
                <span class="font-medium">
                    {format!("Sets to win: {}", generic_config.sets_to_win)}
                </span>
                <span class="hidden sm:inline text-base-content/30">"|"</span>
                <span class="font-medium">{generic_config.display_score_limit()}</span>
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
                && let Ok(cfg) = GenericSportConfig::parse_config(json_cfg)
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

        let sets_to_win =
            Signal::derive(move || current_config.with(|cfg| cfg.as_ref().map(|c| c.sets_to_win)));
        let set_sets_to_win = Callback::new(move |sets: Option<u16>| {
            if let Some(mut cfg) = current_config.get() {
                cfg.sets_to_win = sets.unwrap_or_default();
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        let score_to_win = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().and_then(|c| c.score_to_win))
        });
        let set_score_to_win = Callback::new(move |score: Option<u16>| {
            if let Some(mut cfg) = current_config.get() {
                cfg.score_to_win = score;
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        let win_by_margin = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().and_then(|c| c.win_by_margin))
        });
        let set_win_by_margin = Callback::new(move |margin: Option<u16>| {
            if let Some(mut cfg) = current_config.get() {
                cfg.win_by_margin = margin;
                sport_config_editor
                    .set_config
                    .set(serde_json::to_value(cfg).unwrap());
            }
        });
        let hard_cap = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().and_then(|c| c.hard_cap))
        });
        let set_hard_cap = Callback::new(move |cap: Option<u16>| {
            if let Some(mut cfg) = current_config.get() {
                cfg.hard_cap = cap;
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
        let expected_match_duration_minutes = Signal::derive(move || {
            current_config.with(|cfg| cfg.as_ref().map(|c| c.expected_match_duration_minutes))
        });
        let set_expected_match_duration_minutes =
            Callback::new(move |duration: Option<Duration>| {
                if let Some(mut cfg) = current_config.get() {
                    cfg.expected_match_duration_minutes =
                        duration.unwrap_or(Duration::from_secs(0));
                    sport_config_editor
                        .set_config
                        .set(serde_json::to_value(cfg).unwrap());
                }
            });

        view! {
            <div class="space-y-4" data-testid="sport-config-configuration">
                <NumberInputWithValidation
                    label="Sets to Win"
                    name="sets_to_win"
                    data_testid="input-sets_to_win"
                    value=sets_to_win
                    set_value=set_sets_to_win
                    validation_result=validation_result
                    object_id=sport_config_editor.sport_config_id
                    field="sets_to_win"
                    min="1"
                />
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
                    label="Expected Match Duration"
                    name="expected_match_duration_minutes"
                    data_testid="input-expected_match_duration_minutes"
                    value=expected_match_duration_minutes
                    set_value=set_expected_match_duration_minutes
                    validation_result=validation_result
                    object_id=sport_config_editor.sport_config_id
                    field="expected_match_duration_minutes"
                    unit=DurationInputUnit::Minutes
                />
            </div>
        }
        .into_any()
    }
}
