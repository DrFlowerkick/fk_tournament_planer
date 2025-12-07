//! Generic implementation of sport port
//! This sport plugin may be used as template for new sport plugins or
//! if no specific sport plugin is available.

pub mod config;
pub mod sport_port;
pub mod sport_web_ui;

use app_core::{
    Match, SportConfig, SportError, SportResult,
    utils::{
        id_version::{IdVersion, VersionId},
        namespace::project_namespace,
    },
};
use config::GenericSportConfig;
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
/// use app_core::utils::id_version::{IdVersion, VersionId};
/// use serde_json::json;
/// use uuid::Uuid;
///
/// let plugin = GenericSportPlugin::new();
/// let sport_id = plugin.get_id_version().get_id().unwrap();
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
    fn id(&self) -> Uuid {
        // The generic sport plugin must use a fixed UUID.
        let sport_name = "generic_sport";
        Uuid::new_v5(&project_namespace(), sport_name.as_bytes())
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

impl VersionId for GenericSportPlugin {
    fn get_id_version(&self) -> IdVersion {
        IdVersion::new(self.id(), 0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use app_core::{SportPort, utils::id_version::IdVersion};
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
