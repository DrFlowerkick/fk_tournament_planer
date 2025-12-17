use app_core::{
    SportError, SportResult,
    utils::validation::{FieldError, ValidationErrors, ValidationResult},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Configuration for the Generic Sport Plugin
///
/// Example configurations for GenericSportConfig
/// Default implementation provides a basic valid configuration for sports like soccer or basketball.
///
/// Volleyball example configuration:
/// ```json
/// {
///     "sets_to_win": 3,
///     "score_to_win": 25,
///     "win_by_margin": 2,
///     "hard_cap": 30,
///     "victory_points_win": 1.0,
///     "victory_points_draw": 0.0,
///     "expected_match_duration_minutes": { "secs": 1800, "nanos": 0 }
/// }
/// ```
///
/// Table Tennis example configuration:
/// ```json
/// {
///     "sets_to_win": 3,
///     "score_to_win": 11,
///     "win_by_margin": 2,
///     "hard_cap": 15,
///     "victory_points_win": 1.0,
///     "victory_points_draw": 0.0,
///     "expected_match_duration_minutes": { "secs": 1200, "nanos": 0 }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericSportConfig {
    /// number of sets to win (min 1)
    /// if > 1, score_to_win must be Some()
    pub sets_to_win: u16,
    /// optional score to win a set
    /// must be Some(), if sets_to_win > 1
    /// set to None, if there is no score limit
    pub score_to_win: Option<u16>,
    /// margin by which winner entrant must have more points than it's opponent
    pub win_by_margin: Option<u16>,
    /// hard cap of score to win a set
    pub hard_cap: Option<u16>,
    /// victory points gained by a win
    pub victory_points_win: f32,
    /// victory points gained by a draw
    pub victory_points_draw: f32,
    /// expected maximum duration of a match in minutes
    pub expected_match_duration_minutes: Duration,
}

impl Default for GenericSportConfig {
    fn default() -> Self {
        Self {
            sets_to_win: 1,
            score_to_win: None,
            win_by_margin: None,
            hard_cap: None,
            victory_points_win: 1.0,
            victory_points_draw: 0.5,
            expected_match_duration_minutes: Duration::from_secs(30 * 60),
        }
    }
}

impl GenericSportConfig {
    pub fn parse_config(config: Value) -> SportResult<Self> {
        match serde_json::from_value(config) {
            Ok(sc) => Ok(sc),
            Err(e) => Err(SportError::InvalidJsonConfig(format!(
                "Failed to parse GenericSportConfig: {}",
                e
            ))),
        }
    }
    pub fn validate(&self, mut errs: ValidationErrors) -> ValidationResult<()> {
        // Basic validation logic
        if self.sets_to_win == 0 {
            errs.add(
                FieldError::builder()
                    .set_field("sets_to_win")
                    .add_user_defined_code("invalid_value")
                    .add_message("sets_to_win must be at least 1")
                    .build(),
            );
        }
        if self.sets_to_win > 1 && self.score_to_win.is_none() {
            errs.add(
                FieldError::builder()
                    .set_field("score_to_win")
                    .add_user_defined_code("invalid_value")
                    .add_message("score_to_win must be set if sets_to_win > 1")
                    .build(),
            );
        }
        if self.win_by_margin.is_some() && self.score_to_win.is_none() {
            errs.add(
                FieldError::builder()
                    .set_field("win_by_margin")
                    .add_user_defined_code("invalid_value")
                    .add_message("win_by_margin cannot be set if score_to_win is None")
                    .build(),
            );
        }
        if self.hard_cap.is_some() && self.score_to_win.is_none() {
            errs.add(
                FieldError::builder()
                    .set_field("hard_cap")
                    .add_user_defined_code("invalid_value")
                    .add_message("hard_cap cannot be set if score_to_win is None")
                    .build(),
            );
        }
        if self.win_by_margin.is_some() && self.hard_cap.is_none() {
            errs.add(
                FieldError::builder()
                    .set_field("hard_cap")
                    .add_user_defined_code("invalid_value")
                    .add_message("hard_cap must be set if win_by_margin is set")
                    .build(),
            );
        }
        if self.hard_cap.is_some() && self.win_by_margin.is_none() {
            errs.add(
                FieldError::builder()
                    .set_field("win_by_margin")
                    .add_user_defined_code("invalid_value")
                    .add_message("win_by_margin must be set if hard_cap is set")
                    .build(),
            );
        }
        if self.victory_points_win < 0.0 {
            errs.add(
                FieldError::builder()
                    .set_field("victory_points_win")
                    .add_user_defined_code("invalid_value")
                    .add_message("victory_points_win cannot be negative")
                    .build(),
            );
        }
        if self.victory_points_draw < 0.0 {
            errs.add(
                FieldError::builder()
                    .set_field("victory_points_draw")
                    .add_user_defined_code("invalid_value")
                    .add_message("victory_points_draw cannot be negative")
                    .build(),
            );
        }
        if self.victory_points_win < self.victory_points_draw {
            errs.add(
                FieldError::builder()
                    .set_field("victory_points_win")
                    .add_user_defined_code("invalid_value")
                    .add_message(
                        "victory_points_win must be greater than or equal to victory_points_draw",
                    )
                    .build(),
            );
        }
        if self.expected_match_duration_minutes.as_secs() == 0 {
            errs.add(
                FieldError::builder()
                    .set_field("expected_match_duration_minutes")
                    .add_user_defined_code("invalid_value")
                    .add_message("expected_match_duration_minutes must be greater than 0")
                    .build(),
            );
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}
