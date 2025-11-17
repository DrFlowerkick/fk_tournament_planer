// match of tournament

use crate::ScheduledEntrant;
use chrono::{DateTime, Local};
use uuid::Uuid;

/// match of tournament
// ToDo: remove allow(dead_code) flag
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Match {
    /// id of match in tournament
    id: Uuid,
    /// id of round
    round_id: Uuid,
    /// number of match
    number: usize,
    /// entrant a, either scheduled or concrete id
    side_a: ScheduledEntrant,
    /// entrant b, either scheduled or concrete id
    side_b: ScheduledEntrant,
    /// station of match
    station: u16,
    /// date and start time of match
    start_at: DateTime<Local>,
    /// We use a Vec for scoring, since some sports score over multiple sets,
    /// e.g. best out of 3 sets
    /// score of a; each Vec entry represents one set
    score_a: Vec<u16>,
    /// score of b; each Vec entry represents one set
    score_b: Vec<u16>,
}
