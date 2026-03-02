//! Implementation of SportPort for Generic Sport Plugin

use super::{DdcSportPlugin, config::DdcSportConfig};
use app_core::{
    EntrantGroupScore, Match, SportConfig, SportError, SportPort, SportResult,
    utils::validation::{ValidationErrors, ValidationResult},
};
use serde_json::Value;
use std::time::Duration;
use uuid::Uuid;

impl SportPort for DdcSportPlugin {
    fn name(&self) -> &'static str {
        "Double Disc Court (DDC)"
    }
    fn get_default_config(&self) -> Value {
        serde_json::to_value(DdcSportConfig::default()).unwrap()
    }
    fn estimate_match_duration(&self, config: &SportConfig) -> SportResult<Duration> {
        let generic_config = self.validate_config(config, ValidationErrors::new())?;
        Ok(generic_config.estimate_match_duration())
    }
    fn validate_config_values(
        &self,
        config: &SportConfig,
        errs: ValidationErrors,
    ) -> ValidationResult<()> {
        self.validate_config(config, errs)?;
        Ok(())
    }

    /// Validates a final score against the rules defined in the configuration.
    /// For sports with multiple sets, each set score is validated.
    /// For sports with multiple sets, only one score point per turn is expected.
    /// Therefore constraints like win_by_margin and hard_cap may be reached, but not exceeded.
    fn validate_final_score(&self, config: &SportConfig, score: &Match) -> SportResult<()> {
        if score.get_sport_id() != &self.id() {
            return Err(SportError::InvalidScore(
                "Match sport_id does not match DdcSportPlugin id".to_string(),
            ));
        }
        if score.get_entrants().is_none() {
            return Err(SportError::InvalidScore(
                "Both sides of the match must have concrete entrant IDs".to_string(),
            ));
        }
        let generic_config = self.validate_config(config, ValidationErrors::new())?;
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
        let generic_config = self.validate_config(config, ValidationErrors::new())?;
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
