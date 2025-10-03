// scoring policy

use uuid::Uuid;

/// There are several kinds of scoring systems in sport. MatchScoring a general
/// purpose data type to satisfy a broad range of different sports.
/// Scoring describes how score points are collected and an optional end of match
/// scoring condition.
/// It describes although the number of victory points for a win or draw and
/// the number of score points for a free ticket in Swiss system.
#[derive(Debug, Clone)]
pub struct ScoringPolicy {
    /// id of scoring policy
    id: Uuid,
    /// name of scoring policy
    name: String,
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
    victory_points_win: f32,
    /// victory points gained by a draw
    victory_points_draw: f32,
    /// score points gained by a free ticket (see Swiss system)
    /// Free ticket wins score_free_ticket to 0
    score_free_ticket: u16,
}

// ToDo: macht es Sinn, von Anfang an die scoring policy als trait zu definieren?
// ToDo: zeit pro Punkt für Zeitabschätzung -> idealerweise gebe ich hierfür meinen Sport an und da sind Werte hinterlegt.
// ToDo: Sport trait definieren
