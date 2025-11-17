// stage of a tournament

use uuid::Uuid;

/// stage of a tournament
// ToDo: remove allow(dead_code) flag
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Stage {
    /// id of stage in tournament
    id: Uuid,
    /// id of tournament
    tournament_id: Uuid,
    /// scheduled stage number
    number: usize,
    /// groups of stage
    groups: Vec<Uuid>,
}
