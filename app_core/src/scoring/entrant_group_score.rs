// entrant group scoring

use uuid::Uuid;

/// EntrantGroupScore is used to collect the total score of an entrant over
/// all matches of one group. Together with TieBreakerPolicy this is used to
/// rank entrants within a group.
/// This data is not saved to database, but calculated on the fly when ranking
/// entrants within a group.
///
// ToDo: remove allow(dead_code) flag
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EntrantGroupScore {
    /// id of entrant
    pub entrant_id: Uuid,
    /// id of group
    pub group_id: Uuid,
    /// achieved victory points
    pub victory_points: f32,
    /// relative score over all matches, e.g.:
    /// - if entrant won 15:11, relative score of this match is 4
    /// - if entrant lost 9:21, relativ score of this match is -12
    ///
    /// relative_score is sum over all matches (-8 for above examples)
    pub relative_score: i16,
    /// total own score points over all matches
    pub total_score: u16,
}

impl EntrantGroupScore {
    /// Creates a new EntrantGroupScore with zeroed scores.
    pub fn new(entrant_id: Uuid, group_id: Uuid) -> Self {
        EntrantGroupScore {
            entrant_id,
            group_id,
            victory_points: 0.0,
            relative_score: 0,
            total_score: 0,
        }
    }
}
