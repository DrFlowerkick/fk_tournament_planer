// round of matches in a group

use uuid::Uuid;

/// round of matches
// ToDo: remove allow(dead_code) flag
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Round {
    /// id of round in tournament
    id: Uuid,
    /// id of group
    group_id: Uuid,
    /// scheduled round number
    number: usize,
    /// matches of round, referenced by id, sorted by match number
    matches: Vec<Uuid>,
    /// entrant which has to pause this round
    paused_entrant: PausedEntrant,
}

/// if odd number of entrants, one entrant must pause this round
#[derive(Debug, Clone)]
pub enum PausedEntrant {
    /// index of scheduled entrant to pause in entrants of group sorted by initial rank
    Scheduled(usize),
    /// id of entrant to pause
    Orchestrated(Uuid),
    /// No entrant to pause
    None,
}
