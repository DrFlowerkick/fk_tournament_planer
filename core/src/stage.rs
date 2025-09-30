// stage of a tournament

use uuid::Uuid;

/// stage of a tournament
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