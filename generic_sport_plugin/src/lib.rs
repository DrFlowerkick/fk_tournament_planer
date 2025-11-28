//! Generic implementation of sport port
//! This sport plugin may be used as template for new sport plugins or
//! if no specific sport plugin is available.

use app_core::{
    EntrantGroupScore, Match, SportConfig, SportError, SportPort, SportResult,
    utils::namespace::project_namespace,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

// ToDo: leptos component to configure generic sport config

/// A generic implementation of the `SportPort`, which may be used,
/// if a specific sport plugin is not available.
///
/// # Examples
///
/// Basic usage with a Soccer-like configuration:
///
/// ```
/// use generic_sport_plugin::GenericSportPlugin;
/// use app_core::{SportPort, SportConfig, Match};
/// use app_core::utils::id_version::IdVersion;
/// use serde_json::json;
/// use uuid::Uuid;
///
/// let plugin = GenericSportPlugin::new();
/// let sport_id = plugin.id();
///
/// let config_json = json!({
///     "sets_to_win": 1,
///     "score_to_win": null,
///     "victory_points_win": 3.0,
///     "victory_points_draw": 1.0,
///     "score_free_ticket": 3,
///     "expected_match_duration_minutes": { "secs": 5400, "nanos": 0 }
/// });
///
/// let config = SportConfig {
///     id_version: IdVersion::new(Uuid::new_v4(), 1),
///     sport_id,
///     name: "Soccer".to_string(),
///     config: config_json,
/// };
///
/// // Validate configuration
/// assert!(plugin.validate_config_values(&config).is_ok());
///
/// // Validate a valid score (2:1)
/// let match_score = Match::new_played(
///     Uuid::new_v4(),
///     Uuid::new_v4(),
///     Uuid::new_v4(),
///     sport_id,
///     vec![2],
///     vec![1],
/// );
/// assert!(plugin.validate_final_score(&config, &match_score).is_ok());
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct GenericSportPlugin {}

impl GenericSportPlugin {
    pub fn new() -> Self {
        Self {}
    }
    fn validate_config(&self, config: &SportConfig) -> SportResult<GenericSportConfig> {
        if config.sport_id != self.id() {
            return Err(SportError::InvalidConfig(
                "SportConfig sport_id does not match GenericSportPlugin id".to_string(),
            ));
        }
        let generic_config = GenericSportConfig::parse_config(config)?;
        generic_config.validate()?;
        Ok(generic_config)
    }
    fn validate_final_score_internal(
        &self,
        config: &GenericSportConfig,
        score: &Match,
    ) -> SportResult<()> {
        let (score_a, score_b) = score.get_scores();
        if score_a.len() != config.sets_to_win as usize
            || score_b.len() != config.sets_to_win as usize
        {
            return Err(SportError::InvalidScore(
                "Score does not have the correct number of sets".to_string(),
            ));
        }
        if let Some(score_to_win) = config.score_to_win {
            for (&a, &b) in score_a.iter().zip(score_b.iter()) {
                if a < score_to_win && b < score_to_win {
                    return Err(SportError::InvalidScore(
                        "Neither entrant reached the score to win".to_string(),
                    ));
                }
                if let Some(margin) = config.win_by_margin
                    && (a as i32 - b as i32).abs() < margin as i32
                {
                    return Err(SportError::InvalidScore(
                        "Winning margin not achieved".to_string(),
                    ));
                }
                if let Some(margin) = config.win_by_margin
                    && (a > score_to_win || b > score_to_win)
                    && (a as i32 - b as i32).abs() > margin as i32
                {
                    return Err(SportError::InvalidScore(
                        "Score exceeds winning margin".to_string(),
                    ));
                }
                if let Some(hard_cap) = config.hard_cap
                    && (a > hard_cap || b > hard_cap)
                {
                    return Err(SportError::InvalidScore(
                        "Score exceeds hard cap".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl SportPort for GenericSportPlugin {
    fn id(&self) -> Uuid {
        // The generic sport plugin must use a fixed UUID.
        let sport_name = "generic_sport";
        Uuid::new_v5(&project_namespace(), sport_name.as_bytes())
    }
    fn name(&self) -> &'static str {
        "Generic Sport"
    }
    fn get_default_config(&self) -> Value {
        serde_json::to_value(GenericSportConfig::default()).unwrap()
    }
    fn estimate_match_duration(&self, config: &SportConfig) -> SportResult<Duration> {
        let generic_config = self.validate_config(config)?;
        Ok(generic_config.expected_match_duration_minutes)
    }
    fn validate_config_values(&self, config: &SportConfig) -> SportResult<()> {
        self.validate_config(config)?;
        Ok(())
    }

    /// Validates a final score against the rules defined in the configuration.
    /// For sports with multiple sets, each set score is validated.
    /// For sports with multiple sets, only one score point per turn is expected.
    /// Therefore constraints like win_by_margin and hard_cap may be reached, but not exceeded.
    fn validate_final_score(&self, config: &SportConfig, score: &Match) -> SportResult<()> {
        if score.get_sport_id() != &self.id() {
            return Err(SportError::InvalidScore(
                "Match sport_id does not match GenericSportPlugin id".to_string(),
            ));
        }
        if score.get_entrants().is_none() {
            return Err(SportError::InvalidScore(
                "Both sides of the match must have concrete entrant IDs".to_string(),
            ));
        }
        let generic_config = self.validate_config(config)?;
        self.validate_final_score_internal(&generic_config, score)?;
        Ok(())
    }

    /// Gathers and calculates entrant group score
    fn get_entrant_group_score(
        &self,
        config: &SportConfig,
        group_id: Uuid,
        entrant_id: Uuid,
        all_matches: &[Match],
    ) -> SportResult<EntrantGroupScore> {
        let generic_config = self.validate_config(config)?;
        let mut group_score = EntrantGroupScore::new(entrant_id, group_id);
        for m in all_matches.iter().filter(|m| {
            if let Some((id_a, id_b)) = m.get_entrants() {
                (id_a == &entrant_id || id_b == &entrant_id)
                    && m.get_group_id() == &group_id
                    && m.is_played()
            } else {
                false
            }
        }) {
            // unwrap is safe due to filter
            let (id_a, _id_b) = m.get_entrants().unwrap();
            let entrant_is_a = id_a == &entrant_id;
            let (score_a, score_b) = m.get_scores();
            let entrant_score = if entrant_is_a { score_a } else { score_b };
            let opponent_score = if entrant_is_a { score_b } else { score_a };
            let mut sets_won = 0;
            let mut sets_lost = 0;
            for (&a, &b) in entrant_score.iter().zip(opponent_score.iter()) {
                if a > b {
                    sets_won += 1;
                } else if b > a {
                    sets_lost += 1;
                }
                group_score.total_score += a;
                group_score.relative_score += a as i16 - b as i16;
            }
            if sets_won > sets_lost {
                group_score.victory_points += generic_config.victory_points_win;
            } else if sets_won == sets_lost {
                group_score.victory_points += generic_config.victory_points_draw;
            }
        }
        Ok(group_score)
    }
}

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
///     "score_free_ticket": 8,
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
///     "score_free_ticket": 5,
///     "expected_match_duration_minutes": { "secs": 1200, "nanos": 0 }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericSportConfig {
    /// number of sets to win (min 1)
    /// if > 1, score_to_win must be Some()
    sets_to_win: u16,
    /// optional score to win a set
    /// must be Some(), if sets_to_win > 1
    /// set to None, if there is no score limit
    score_to_win: Option<u16>,
    /// margin by which winner entrant must have more points than it's opponent
    win_by_margin: Option<u16>,
    /// hard cap of score to win a set
    hard_cap: Option<u16>,
    /// victory points gained by a win
    victory_points_win: f32,
    /// victory points gained by a draw
    victory_points_draw: f32,
    /// score points gained by a free ticket (see Swiss system)
    /// Free ticket wins score_free_ticket to 0
    score_free_ticket: u16,
    /// expected maximum duration of a match in minutes
    expected_match_duration_minutes: Duration,
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
            score_free_ticket: 1,
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
        if self.score_free_ticket == 0 {
            return Err(SportError::InvalidConfig(
                "score_free_ticket must be at least 1".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::utils::id_version::IdVersion;
    use serde_json::json;

    #[test]
    fn test_validate_config_soccer() {
        let plugin = GenericSportPlugin::new();
        let config = json!({
            "sets_to_win": 1,
            "score_to_win": null,
            "win_by_margin": null,
            "hard_cap": null,
            "victory_points_win": 3.0,
            "victory_points_draw": 1.0,
            "score_free_ticket": 1,
            "expected_match_duration_minutes": { "secs": 5400, "nanos": 0 }
        });
        let sport_config = SportConfig {
            id_version: IdVersion::new(Uuid::new_v4(), 1),
            sport_id: plugin.id(),
            name: "Soccer".to_string(),
            config,
        };
        assert!(plugin.validate_config(&sport_config).is_ok());
    }

    #[test]
    fn test_validate_config_volleyball() {
        let plugin = GenericSportPlugin::new();
        let config = json!({
            "sets_to_win": 3,
            "score_to_win": 25,
            "win_by_margin": 2,
            "hard_cap": 30,
            "victory_points_win": 1.0,
            "victory_points_draw": 0.0,
            "score_free_ticket": 8,
            "expected_match_duration_minutes": { "secs": 1800, "nanos": 0 }
        });
        let sport_config = SportConfig {
            id_version: IdVersion::new(Uuid::new_v4(), 1),
            sport_id: plugin.id(),
            name: "Volleyball".to_string(),
            config,
        };
        assert!(plugin.validate_config(&sport_config).is_ok());
    }

    #[test]
    fn test_validate_final_score_soccer() {
        let plugin = GenericSportPlugin::new();
        let config = json!({
            "sets_to_win": 1,
            "score_to_win": null,
            "win_by_margin": null,
            "hard_cap": null,
            "victory_points_win": 3.0,
            "victory_points_draw": 1.0,
            "score_free_ticket": 1,
            "expected_match_duration_minutes": { "secs": 5400, "nanos": 0 }
        });
        let sport_config = SportConfig {
            id_version: IdVersion::new(Uuid::new_v4(), 1),
            sport_id: plugin.id(),
            name: "Soccer".to_string(),
            config,
        };

        let match_score = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![2],
            vec![1],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score)
                .is_ok()
        );

        let match_score_draw = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![1],
            vec![1],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_draw)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_final_score_volleyball() {
        let plugin = GenericSportPlugin::new();
        let config = json!({
            "sets_to_win": 3,
            "score_to_win": 25,
            "win_by_margin": 2,
            "hard_cap": 30,
            "victory_points_win": 1.0,
            "victory_points_draw": 0.0,
            "score_free_ticket": 8,
            "expected_match_duration_minutes": { "secs": 1800, "nanos": 0 }
        });
        let sport_config = SportConfig {
            id_version: IdVersion::new(Uuid::new_v4(), 1),
            sport_id: plugin.id(),
            name: "Volleyball".to_string(),
            config,
        };

        // Valid score 3:0
        let match_score = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![25, 25, 25],
            vec![20, 20, 20],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score)
                .is_ok()
        );

        // Invalid score: not enough sets
        let match_score_invalid_sets = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![25, 25],
            vec![20, 20],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_invalid_sets)
                .is_err()
        );

        // Invalid score: score to win not reached
        let match_score_invalid_points = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![24, 25, 25],
            vec![20, 20, 20],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_invalid_points)
                .is_err()
        );

        // Invalid score: margin not reached (25:24)
        let match_score_invalid_margin = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![25, 25, 25],
            vec![24, 20, 20],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_invalid_margin)
                .is_err()
        );

        // Valid score: margin reached (26:24)
        let match_score_valid_margin = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![26, 25, 25],
            vec![24, 20, 20],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_valid_margin)
                .is_ok()
        );

        // Invalid score: hard cap exceeded (31:29)
        let match_score_hard_cap = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![31, 25, 25],
            vec![29, 20, 20],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_hard_cap)
                .is_err()
        );
    }
}
