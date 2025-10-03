// entrant group scoring

use uuid::Uuid;

/// EntrantGroupScore may be used to collect the total score of an entrant over
/// all matches of one group. It describes although options of comparing of
/// comparing EntrantGroupScore.
#[derive(Debug, Clone)]
pub struct EntrantGroupScore {
    /// id of entrant group score
    id: Uuid,
    /// id of entrant
    entrant_id: Uuid,
    /// id of group
    group_id: Uuid,
    /// achieved victory points
    victory_points: f32,
    /// relative score over all matches, e.g.:
    /// - if entrant won 15:11, relative score of this match is 4
    /// - if entrant lost 9:21, relativ score of this match is -12
    /// relative_score is sum over all matches (-8 for above examples)
    relative_score: i16,
    /// total own score points over all matches
    total_score: u16,
}
