// tie breaker policy

use uuid::Uuid;

/// policy to break ties. Tie breaker rules are resolved in vec order
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
