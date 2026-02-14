use app_core::{
    SportError, SportResult,
    utils::validation::{FieldError, ValidationErrors, ValidationResult},
};
use app_utils::enum_utils::SelectableOption;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Display, time::Duration};
use uuid::Uuid;

/// DdcSetCfg - configuration for sets in Double Disc Court (DDC)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum DdcSetCfg {
    /// Best of 1 set
    #[default]
    BestOf1,
    /// Best of 3 sets
    BestOf3,
    /// Best of 5 sets
    BestOf5,
    /// Custom number of sets
    CustomSetsToWin { sets_to_win: u16 },
    /// Custom sets to play; if even, match may end in a draw
    CustomTotalSets { total_sets: u16 },
}

impl Display for DdcSetCfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdcSetCfg::BestOf1 => write!(f, "Sets to win: 1"),
            DdcSetCfg::BestOf3 => write!(f, "Sets to win: 2"),
            DdcSetCfg::BestOf5 => write!(f, "Sets to win: 3"),
            DdcSetCfg::CustomSetsToWin { sets_to_win } => {
                write!(f, "Custom sets to win: {}", sets_to_win)
            }
            DdcSetCfg::CustomTotalSets { total_sets } => {
                write!(f, "Custom total sets: {}", total_sets)
            }
        }
    }
}

impl SelectableOption for DdcSetCfg {
    fn value(&self) -> String {
        self.to_string()
    }

    fn label(&self) -> String {
        self.to_string()
    }

    fn options(&self) -> Vec<Self> {
        match self {
            DdcSetCfg::CustomSetsToWin { sets_to_win } => vec![
                DdcSetCfg::BestOf1,
                DdcSetCfg::BestOf3,
                DdcSetCfg::BestOf5,
                DdcSetCfg::CustomSetsToWin {
                    sets_to_win: *sets_to_win,
                },
                DdcSetCfg::CustomTotalSets { total_sets: 0 },
            ],
            DdcSetCfg::CustomTotalSets { total_sets } => vec![
                DdcSetCfg::BestOf1,
                DdcSetCfg::BestOf3,
                DdcSetCfg::BestOf5,
                DdcSetCfg::CustomSetsToWin { sets_to_win: 0 },
                DdcSetCfg::CustomTotalSets {
                    total_sets: *total_sets,
                },
            ],
            _ => vec![
                DdcSetCfg::BestOf1,
                DdcSetCfg::BestOf3,
                DdcSetCfg::BestOf5,
                DdcSetCfg::CustomSetsToWin { sets_to_win: 0 },
                DdcSetCfg::CustomTotalSets { total_sets: 0 },
            ],
        }
    }

    fn static_options() -> Vec<Self> {
        Self::options(&Self::default())
    }
}

impl DdcSetCfg {
    /// validates the set configuration
    pub fn validate(&self, object_id: Uuid, mut errs: ValidationErrors) -> ValidationErrors {
        match self {
            DdcSetCfg::CustomSetsToWin { sets_to_win } => {
                if *sets_to_win == 0 {
                    errs.add(
                        FieldError::builder()
                            .set_field("sets_cfg")
                            .add_user_defined_code("invalid_value")
                            .add_message("sets_to_win must be at least 1")
                            .set_object_id(object_id)
                            .build(),
                    );
                }
            }
            DdcSetCfg::CustomTotalSets { total_sets } => {
                if *total_sets == 0 {
                    errs.add(
                        FieldError::builder()
                            .set_field("sets_cfg")
                            .add_user_defined_code("invalid_value")
                            .add_message("total_sets must be at least 1")
                            .set_object_id(object_id)
                            .build(),
                    );
                }
            }
            _ => {}
        }
        errs
    }

    /// Returns the minimum and maximum number of sets to play.
    pub fn sets_to_play(&self) -> (u16, u16) {
        match self {
            DdcSetCfg::BestOf1 => (1, 1),
            DdcSetCfg::BestOf3 => (2, 3),
            DdcSetCfg::BestOf5 => (3, 5),
            DdcSetCfg::CustomSetsToWin { sets_to_win } => {
                (*sets_to_win, (*sets_to_win * 2).saturating_sub(1))
            }
            DdcSetCfg::CustomTotalSets { total_sets } => (*total_sets, *total_sets),
        }
    }
}

/// DdcSetWinningCfg – configuration for winning a single set in Double Disc Court (DDC)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum DdcSetWinningCfg {
    /// Score 11 to win a set, hard cap at 15, win by margin 2
    #[default]
    Sw11Hc15M2,
    /// Score 15 to win a set, hard cap at 21, win by margin 2
    Sw15Hc21M2,
    /// Score 21 to win a set, hard cap at 25, win by margin 2
    Sw21Hc25M2,
    /// Custom set configuration
    Custom {
        score_to_win: u16,
        win_by_margin: u16,
        hard_cap: u16,
    },
}

impl Display for DdcSetWinningCfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdcSetWinningCfg::Sw11Hc15M2 => {
                write!(f, "Score: 11 (+2, Cap 15)")
            }
            DdcSetWinningCfg::Sw15Hc21M2 => {
                write!(f, "Score: 15 (+2, Cap 21)")
            }
            DdcSetWinningCfg::Sw21Hc25M2 => {
                write!(f, "Score: 21 (+2, Cap 25)")
            }
            DdcSetWinningCfg::Custom {
                score_to_win,
                win_by_margin,
                hard_cap,
            } => write!(
                f,
                "Custom score: {} (+{}, Cap {})",
                score_to_win, win_by_margin, hard_cap
            ),
        }
    }
}

impl SelectableOption for DdcSetWinningCfg {
    fn value(&self) -> String {
        self.to_string()
    }

    fn label(&self) -> String {
        self.to_string()
    }

    fn options(&self) -> Vec<Self> {
        match self {
            DdcSetWinningCfg::Custom {
                score_to_win,
                win_by_margin,
                hard_cap,
            } => vec![
                DdcSetWinningCfg::Sw11Hc15M2,
                DdcSetWinningCfg::Sw15Hc21M2,
                DdcSetWinningCfg::Sw21Hc25M2,
                DdcSetWinningCfg::Custom {
                    score_to_win: *score_to_win,
                    win_by_margin: *win_by_margin,
                    hard_cap: *hard_cap,
                },
            ],
            _ => vec![
                DdcSetWinningCfg::Sw11Hc15M2,
                DdcSetWinningCfg::Sw15Hc21M2,
                DdcSetWinningCfg::Sw21Hc25M2,
                DdcSetWinningCfg::Custom {
                    score_to_win: 0,
                    win_by_margin: 0,
                    hard_cap: 0,
                },
            ],
        }
    }

    fn static_options() -> Vec<Self> {
        Self::options(&Self::default())
    }
}

impl DdcSetWinningCfg {
    pub fn get_win_cfg(&self) -> (u16, u16, u16) {
        match self {
            DdcSetWinningCfg::Sw11Hc15M2 => (11, 2, 15),
            DdcSetWinningCfg::Sw15Hc21M2 => (15, 2, 21),
            DdcSetWinningCfg::Sw21Hc25M2 => (21, 2, 25),
            DdcSetWinningCfg::Custom {
                score_to_win,
                win_by_margin,
                hard_cap,
            } => (*score_to_win, *win_by_margin, *hard_cap),
        }
    }

    /// Validates the set winning configuration
    pub fn validate(&self, object_id: Uuid, mut errs: ValidationErrors) -> ValidationErrors {
        match self {
            DdcSetWinningCfg::Custom {
                score_to_win,
                win_by_margin,
                hard_cap,
            } => {
                if *score_to_win == 0 {
                    errs.add(
                        FieldError::builder()
                            .set_field("score_to_win")
                            .add_user_defined_code("invalid_value")
                            .add_message("score_to_win must be at least 1")
                            .set_object_id(object_id)
                            .build(),
                    );
                }
                if *score_to_win < *win_by_margin {
                    errs.add(
                        FieldError::builder()
                            .set_field("score_to_win")
                            .add_user_defined_code("invalid_value")
                            .add_message(
                                "score_to_win must be greater than or equal to win_by_margin",
                            )
                            .set_object_id(object_id)
                            .build(),
                    );
                }
                if *win_by_margin == 0 {
                    errs.add(
                        FieldError::builder()
                            .set_field("win_by_margin")
                            .add_user_defined_code("invalid_value")
                            .add_message("win_by_margin must be at least 1")
                            .set_object_id(object_id)
                            .build(),
                    );
                }
                if *hard_cap <= *score_to_win + *win_by_margin {
                    errs.add(
                        FieldError::builder()
                            .set_field("hard_cap")
                            .add_user_defined_code("invalid_value")
                            .add_message(
                                "hard_cap must be greater than score_to_win plus win_by_margin",
                            )
                            .set_object_id(object_id)
                            .build(),
                    );
                }
            }
            _ => {}
        }
        errs
    }

    pub fn validate_final_set_score(&self, score_a: u16, score_b: u16) -> SportResult<()> {
        let (score_to_win, win_by_margin, hard_cap) = self.get_win_cfg();
        let max_score = score_a.max(score_b);
        let min_score = score_a.min(score_b);
        if max_score < score_to_win {
            return Err(SportError::InvalidScore(
                "No entrant has reached the score to win".to_string(),
            ));
        }
        // DDC specific: with a double point in the last rally, the score may exceed the hard cap by 1
        if max_score > hard_cap + 1 {
            return Err(SportError::InvalidScore(
                "Score exceeds hard cap".to_string(),
            ));
        }
        if max_score < hard_cap && max_score - min_score < win_by_margin {
            return Err(SportError::InvalidScore(
                "Winning margin not reached".to_string(),
            ));
        }
        Ok(())
    }

    pub fn max_num_rallies_without_hc_and_doubles(&self) -> u16 {
        let (score_to_win, win_by_margin) = match self {
            DdcSetWinningCfg::Sw11Hc15M2 => (11, 2),
            DdcSetWinningCfg::Sw15Hc21M2 => (15, 2),
            DdcSetWinningCfg::Sw21Hc25M2 => (21, 2),
            DdcSetWinningCfg::Custom {
                score_to_win,
                win_by_margin,
                hard_cap: _,
            } => (*score_to_win, *win_by_margin),
        };
        // Maximum rallies without exceeding hard cap and no doubles
        // For example, if score_to_win is 15 and win_by_margin is 2,
        // the maximum result without exceeding the hard cap is 15 (winner) to 13 (opponent).
        // With one point per rally, this results in 28 played rallies.
        score_to_win + (score_to_win - win_by_margin)
    }
}

/// Configuration for the Double Disc Court (DDC) Plugin
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DdcSportConfig {
    /// configuration for sets
    pub sets_cfg: DdcSetCfg,
    /// configuration for winning a set
    pub set_winning_cfg: DdcSetWinningCfg,
    /// victory points gained by a win
    pub victory_points_win: f32,
    /// victory points gained by a draw
    pub victory_points_draw: f32,
    /// expected mean duration of a rally in seconds
    /// Used to estimate match duration by multiplying with max number of rallies
    /// without exceeding hard cap and no doubles.
    /// For example, if score_to_win is 15 and win_by_margin is 2,
    /// the maximum result without exceeding the hard cap is 15 (winner) to 13 (opponent).
    /// With one point per rally, this results in 28 played rallies.
    pub expected_rally_duration_seconds: Duration,
}

impl Default for DdcSportConfig {
    fn default() -> Self {
        Self {
            sets_cfg: DdcSetCfg::BestOf1,
            set_winning_cfg: DdcSetWinningCfg::Sw15Hc21M2,
            victory_points_win: 1.0,
            victory_points_draw: 0.5,
            expected_rally_duration_seconds: Duration::from_secs(45),
        }
    }
}

impl DdcSportConfig {
    pub fn parse_config(config: Value) -> SportResult<Self> {
        match serde_json::from_value(config) {
            Ok(sc) => Ok(sc),
            Err(e) => Err(SportError::InvalidJsonConfig(format!(
                "Failed to parse DdcSportConfig: {}",
                e
            ))),
        }
    }
    pub fn validate(&self, object_id: Uuid, errs: ValidationErrors) -> ValidationResult<()> {
        let errs = self.sets_cfg.validate(object_id, errs);
        let mut errs = self.set_winning_cfg.validate(object_id, errs);
        if self.victory_points_win <= 0.0 {
            errs.add(
                FieldError::builder()
                    .set_field("victory_points_win")
                    .add_user_defined_code("invalid_value")
                    .add_message("victory_points_win must be greater than 0")
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if self.victory_points_draw <= 0.0 {
            errs.add(
                FieldError::builder()
                    .set_field("victory_points_draw")
                    .add_user_defined_code("invalid_value")
                    .add_message("victory_points_draw must be greater than 0")
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if self.victory_points_win <= self.victory_points_draw {
            errs.add(
                FieldError::builder()
                    .set_field("victory_points_draw")
                    .add_user_defined_code("invalid_value")
                    .add_message("victory_points_draw must be less than victory_points_win")
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if self.expected_rally_duration_seconds.as_secs() == 0 {
            errs.add(
                FieldError::builder()
                    .set_field("expected_rally_duration_seconds")
                    .add_user_defined_code("invalid_value")
                    .add_message("expected_rally_duration_seconds must be greater than 0")
                    .set_object_id(object_id)
                    .build(),
            );
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
    pub fn estimate_match_duration(&self) -> Duration {
        let max_sets = self.sets_cfg.sets_to_play().1;
        self.expected_rally_duration_seconds
            * (max_sets
                * self
                    .set_winning_cfg
                    .max_num_rallies_without_hc_and_doubles()) as u32
    }
}
