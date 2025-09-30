// match of tournament

use uuid::Uuid;
use time::OffsetDateTime;
use crate::ScheduledEntrant;

/// match of tournament
#[derive(Debug, Clone)]
pub struct Match {
    /// id of match in tournament
    id: Uuid,
    /// if if round
    round_id: Uuid,
    /// number of match
    number: usize,
    /// scheduled entrant a
    scheduled_a: ScheduledEntrant,
    /// scheduled entrant b
    scheduled_b: ScheduledEntrant,
    /// station of match
    station: u16,
    /// date and start time of match
    start_at: OffsetDateTime,
    /// id of entrant a
    side_a: Uuid,
    /// if of entrant b
    side_b: Uuid,
    /// We use a Vec for scoring, since some sports score over multiple sets,
    /// e.g. best out of 3 sets
    /// score of a; each Vec entry represents one set
    score_a: Vec<u16>,
    /// score of b; each Vec entry represents one set
    score_b: Vec<u16>,
}
