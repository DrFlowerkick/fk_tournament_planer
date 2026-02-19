//! Generic implementation of sport port
//! This sport plugin may be used as template for new sport plugins or
//! if no specific sport plugin is available.

pub mod config;
pub mod sport_port;
pub mod sport_web_ui;

use app_core::{
    Match, SportConfig, SportError, SportResult,
    utils::{
        id_version::IdVersion,
        namespace::project_namespace,
        traits::ObjectIdVersion,
        validation::{FieldError, ValidationErrors, ValidationResult},
    },
};
use config::DdcSportConfig;
use uuid::Uuid;

/// Implementation of the `SportPort` for "Double Disc Court (DDC)".
#[derive(Debug, Default, Clone, Copy)]
pub struct DdcSportPlugin {}

impl DdcSportPlugin {
    pub fn new() -> Self {
        Self {}
    }
    fn id(&self) -> Uuid {
        // The generic sport plugin must use a fixed UUID.
        let sport_name = "double_disc_court";
        Uuid::new_v5(&project_namespace(), sport_name.as_bytes())
    }
    fn validate_config(
        &self,
        config: &SportConfig,
        errs: ValidationErrors,
    ) -> ValidationResult<DdcSportConfig> {
        if config.get_sport_id() != self.id() {
            let err = FieldError::builder()
                .set_field("sport_id")
                .add_message(format!(
                    "Sport ID does not match DdcSportPlugin id: expected {}, got {}",
                    self.id(),
                    config.get_sport_id()
                ))
                .set_object_id(config.get_id())
                .build();
            return Err(err.into());
        }
        let generic_config = match DdcSportConfig::parse_config(config.get_config().clone()) {
            Ok(cfg) => cfg,
            Err(e) => {
                let err = FieldError::builder()
                    .set_field("sport_config_json")
                    .add_message(format!("Invalid sport configuration JSON: {}", e))
                    .set_object_id(config.get_id())
                    .build();
                return Err(err.into());
            }
        };
        generic_config.validate(config.get_id(), errs)?;
        Ok(generic_config)
    }
    fn validate_final_score_internal(
        &self,
        config: &DdcSportConfig,
        score: &Match,
    ) -> SportResult<()> {
        let (score_a, score_b) = score.get_scores();
        let (min_sets, max_sets) = config.sets_cfg.sets_to_play();
        if score_a.len() != score_b.len() {
            return Err(SportError::InvalidScore(
                "Score vectors for both entrants must have the same length".to_string(),
            ));
        }
        if !(min_sets as usize..=max_sets as usize).contains(&score_a.len()) {
            return Err(SportError::InvalidScore(
                "Score does not have the correct number of sets".to_string(),
            ));
        }
        for (&a, &b) in score_a.iter().zip(score_b.iter()) {
            config.set_winning_cfg.validate_final_set_score(a, b)?;
        }

        Ok(())
    }
}

impl ObjectIdVersion for DdcSportPlugin {
    fn get_id_version(&self) -> IdVersion {
        // we can increment version later if changes are made to the sport plugin
        IdVersion::new(self.id(), Some(0))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use app_core::{SportPort, utils::id_version::IdVersion};
    use serde_json::json;

    #[test]
    fn test_validate_config_default() {
        let plugin = DdcSportPlugin::new();
        let config = json!({
            "sets_cfg": "BestOf1",
            "set_winning_cfg": "Sw15Hc21M2",
            "victory_points_win": 1.0,
            "victory_points_draw": 0.5,
            "expected_rally_duration_seconds": { "secs": 45, "nanos": 0 }
        });
        let id_version = IdVersion::new(Uuid::new_v4(), Some(1));
        let mut sport_config = SportConfig::new(id_version);
        sport_config
            .set_sport_id(plugin.id())
            .set_name("DDC Default")
            .set_config(config);
        assert!(
            plugin
                .validate_config(&sport_config, ValidationErrors::new())
                .is_ok()
        );
    }

    #[test]
    fn test_validate_config_custom() {
        let plugin = DdcSportPlugin::new();
        let config = json!({
            "sets_cfg": { "CustomSetsToWin": { "sets_to_win": 2 } },
            "set_winning_cfg": { "Custom": { "score_to_win": 15, "win_by_margin": 2, "hard_cap": 20 } },
            "victory_points_win": 2.0,
            "victory_points_draw": 1.0,
            "expected_rally_duration_seconds": { "secs": 30, "nanos": 0 }
        });
        let id_version = IdVersion::new(Uuid::new_v4(), Some(1));
        let mut sport_config = SportConfig::new(id_version);
        sport_config
            .set_sport_id(plugin.id())
            .set_name("DDC Custom")
            .set_config(config);
        assert!(
            plugin
                .validate_config(&sport_config, ValidationErrors::new())
                .is_ok()
        );
    }

    #[test]
    fn test_validate_final_score_default() {
        let plugin = DdcSportPlugin::new();
        // Default: BestOf1, Sw15Hc21M2
        let config = json!({
            "sets_cfg": "BestOf1",
            "set_winning_cfg": "Sw15Hc21M2",
            "victory_points_win": 1.0,
            "victory_points_draw": 0.5,
            "expected_rally_duration_seconds": { "secs": 45, "nanos": 0 }
        });
        let id_version = IdVersion::new(Uuid::new_v4(), Some(1));
        let mut sport_config = SportConfig::new(id_version);
        sport_config
            .set_sport_id(plugin.id())
            .set_name("DDC Default")
            .set_config(config);

        // Valid score 15:13
        let match_score = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![15],
            vec![13],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score)
                .is_ok()
        );

        // Invalid score: margin not reached (15:14)
        let match_score_margin = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![15],
            vec![14],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_margin)
                .is_err()
        );

        // Invalid score: hard cap exceeded (23:20) - Hard Cap is 21
        let match_score_hard_cap = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![23],
            vec![20],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_hard_cap)
                .is_err()
        );
    }

    #[test]
    fn test_validate_final_score_custom() {
        let plugin = DdcSportPlugin::new();
        // Custom: BestOf3 (2 sets to win), Sw11Hc15M2
        let config = json!({
            "sets_cfg": "BestOf3",
            "set_winning_cfg": "Sw11Hc15M2",
            "victory_points_win": 1.0,
            "victory_points_draw": 0.5,
            "expected_rally_duration_seconds": { "secs": 45, "nanos": 0 }
        });
        let id_version = IdVersion::new(Uuid::new_v4(), Some(1));
        let mut sport_config = SportConfig::new(id_version);
        sport_config
            .set_sport_id(plugin.id())
            .set_name("DDC Custom")
            .set_config(config);

        // Valid score 2:0 sets (11:9, 11:5)
        let match_score = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![11, 11],
            vec![9, 5],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score)
                .is_ok()
        );

        // Valid score 2:1 sets (11:9, 5:11, 11:8)
        let match_score_3sets = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![11, 5, 11],
            vec![9, 11, 8],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_3sets)
                .is_ok()
        );

        // Invalid score: not enough sets (1 set played)
        let match_score_invalid_sets = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![11],
            vec![9],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_invalid_sets)
                .is_err()
        );

        // Invalid score: score to win not reached (10:8)
        let match_score_invalid_points = Match::new_played(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            plugin.id(),
            vec![10, 11],
            vec![8, 5],
        );
        assert!(
            plugin
                .validate_final_score(&sport_config, &match_score_invalid_points)
                .is_err()
        );
    }
}
