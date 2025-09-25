// scoring system and constraints

use ordered_float::NotNan;
use std::cmp::Ordering;
use std::collections::HashSet;
use uuid::Uuid;

/// There are several kinds of scoring systems in sport. MatchScoring a general
/// purpose data type to satisfy a broad range of different sports.
/// Scoring describes how score points are collected and an optional end of match
/// scoring condition.
/// It describes although the number of victory points for a win or draw and
/// the number of victory points for a free ticket in Swiss system.
#[derive(Debug, Clone)]
pub struct ScoringPolicy {
    /// number of sets to win (min 1)
    /// if > 1, score_to_win must be Some()
    sets_to_win: u16,
    /// optional score to win a set
    /// must be Some(), if sets_to_win > 1
    score_to_win: Option<u16>,
    /// margin by which winner entrant must have more points than it's opponent
    win_by_margin: Option<u16>,
    /// hard cap of score to win a set
    hard_cap: Option<u16>,
    /// victory points gained by a win
    victory_points_win: NotNan<f32>,
    /// victory points gained by a draw
    victory_points_draw: NotNan<f32>,
    /// victory points gained by a free ticket (see Swiss system)
    victory_points_free_ticket: NotNan<f32>,
}

/// EntrantGroupScore may be used to collect the total score of an entrant over
/// all matches of one group. It describes although options of comparing of
/// comparing EntrantGroupScore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntrantGroupScore {
    /// id of entrant
    entrant_id: Uuid,
    /// achieved victory points
    victory_points: NotNan<f32>,
    /// relative score over all matches, e.g.:
    /// - if entrant won 15:11, relative score of this match is 4
    /// - if entrant lost 9:21, relativ score of this match is -12
    /// relative_score is sum over all matches (-8 for above examples)
    relative_score: i16,
    /// total own score points over all matches
    total_score: u16,
    /// won against opponents is required for direct compare
    won_against_opponents: HashSet<Uuid>,
    /// if swiss system, set after each round swiss scores
    swiss_scoring: Option<SwissScoring>,
    final_compare: FinalScoreCompare,
}

impl Ord for EntrantGroupScore {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.victory_points.cmp(&other.victory_points) {
            Ordering::Equal => {
                if let Some(self_swiss_scoring) = self.swiss_scoring.as_ref()
                    && let Some(other_swiss_scoring) = other.swiss_scoring.as_ref()
                {
                    match self_swiss_scoring.cmp(&other_swiss_scoring) {
                        Ordering::Equal => {}
                        ord => return ord,
                    }
                }
                match self.relative_score.cmp(&other.relative_score) {
                    Ordering::Equal => match self.total_score.cmp(&other.total_score) {
                        Ordering::Equal => {
                            if self.won_against_opponents.contains(&other.entrant_id) {
                                return Ordering::Greater;
                            }
                            if other.won_against_opponents.contains(&self.entrant_id) {
                                return Ordering::Less;
                            }
                            self.final_compare.cmp(&other.final_compare)
                        }
                        ord => ord,
                    },
                    ord => ord,
                }
            }
            ord => ord,
        }
    }
}

impl PartialOrd for EntrantGroupScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Swiss scoring must be calculated after all matches of one round are done. These scores are sum
/// scores of opponents scores, which the entrant has faced in the tournament. See swiss system.
/// These scores describe how strong the faced opponents are. In a tie break the entrant with the
/// stronger opponents wins the tie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwissScoring {
    /// sum of all victory points of faced opponents.
    buchholz_score: NotNan<f32>,
    /// sum of relative score of faced opponents
    sum_opp_relative_score: i16,
    /// sum of total score of faced opponents
    sum_opp_total_score: u16,
}

impl Ord for SwissScoring {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.buchholz_score.cmp(&other.buchholz_score) {
            Ordering::Equal => match self
                .sum_opp_relative_score
                .cmp(&other.sum_opp_relative_score)
            {
                Ordering::Equal => self.sum_opp_total_score.cmp(&other.sum_opp_total_score),
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl PartialOrd for SwissScoring {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalScoreCompare {
    InitialRank(usize),
    CoinFlip,
    Draw,
}

impl Ord for FinalScoreCompare {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                FinalScoreCompare::InitialRank(self_rank),
                FinalScoreCompare::InitialRank(other_rank),
            ) => self_rank.cmp(other_rank),
            (FinalScoreCompare::CoinFlip, FinalScoreCompare::CoinFlip) => {
                if rand::random() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for FinalScoreCompare {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
