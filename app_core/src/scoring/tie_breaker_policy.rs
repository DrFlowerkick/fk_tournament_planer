// tie breaker policy

use uuid::Uuid;

/// policy to break ties. Tie breaker rules are resolved in vec order
/// // ToDo: remove allow(dead_code) flag
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TieBreakerPolicy {
    /// if of policy
    id: Uuid,
    /// name of policy
    name: String,
    /// tie breaker rules sorted from most to least important rule
    tie_breakers: Vec<TieBreaker>,
}

#[derive(Debug, Clone)]
pub enum TieBreaker {
    VictoryPoints,
    BuchholzScore,
    SumOpponentRelativeScore,
    SumOpponentTotalScore,
    RelativScore,
    TotalScore,
    HeadToHead,
    CoinFlip,
    Draw,
}

// es muss irgendein "finales" tiebreaker geben. Aktuell sind das Coin Flip oder Draw -> alternativ Draw ausgeben, wenn kein Tiebreaker zeiht.

/// Contains all data required to resolve tie-breakers for a single entrant.
/// The core ranking logic will use this data in the order specified by a `TieBreakerPolicy`.
#[derive(Debug, Clone, Default)]
pub struct TieBreakerData {
    pub victory_points: f32,
    pub buchholz_score: f32,
    pub sum_opponent_relative_score: i32,
    pub sum_opponent_total_score: u32,
    pub relative_score: i32,
    pub total_score: u32,
    // HeadToHead is resolved by looking at direct matches, not by pre-calculated data.
}