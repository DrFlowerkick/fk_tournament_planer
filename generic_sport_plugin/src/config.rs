use app_core::{SportConfig, SportError, SportResult};
use serde::{Deserialize, Serialize};
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
    pub fn parse_config(config: &SportConfig) -> SportResult<Self> {
        serde_json::from_value(config.config.clone()).map_err(|e| {
            SportError::InvalidConfig(format!("Failed to parse GenericSportConfig: {}", e))
        })
    }
    pub fn validate(&self) -> SportResult<()> {
        // Basic validation logic
        if self.sets_to_win == 0 {
            return Err(SportError::InvalidConfig(
                "sets_to_win must be at least 1".to_string(),
            ));
        }
        if self.sets_to_win > 1 && self.score_to_win.is_none() {
            return Err(SportError::InvalidConfig(
                "score_to_win must be set if sets_to_win > 1".to_string(),
            ));
        }
        if self.win_by_margin.is_some() && self.score_to_win.is_none() {
            return Err(SportError::InvalidConfig(
                "win_by_margin cannot be set if score_to_win is None".to_string(),
            ));
        }
        if self.hard_cap.is_some() && self.score_to_win.is_none() {
            return Err(SportError::InvalidConfig(
                "hard_cap cannot be set if score_to_win is None".to_string(),
            ));
        }
        if self.win_by_margin.is_some() && self.hard_cap.is_none() {
            return Err(SportError::InvalidConfig(
                "hard_cap must be set if win_by_margin is set".to_string(),
            ));
        }
        if self.hard_cap.is_some() && self.win_by_margin.is_none() {
            return Err(SportError::InvalidConfig(
                "win_by_margin must be set if hard_cap is set".to_string(),
            ));
        }
        if self.victory_points_win < 0.0 {
            return Err(SportError::InvalidConfig(
                "victory_points_win cannot be negative".to_string(),
            ));
        }
        if self.victory_points_draw < 0.0 {
            return Err(SportError::InvalidConfig(
                "victory_points_draw cannot be negative".to_string(),
            ));
        }
        if self.victory_points_win < self.victory_points_draw {
            return Err(SportError::InvalidConfig(
                "victory_points_win must be greater than or equal to victory_points_draw"
                    .to_string(),
            ));
        }
        if self.expected_match_duration_minutes.as_secs() == 0 {
            return Err(SportError::InvalidConfig(
                "expected_match_duration_minutes must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}
