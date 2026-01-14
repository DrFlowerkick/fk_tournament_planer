// stage of a tournament

use crate::utils::{
    id_version::IdVersion,
    traits::{ObjectIdVersion, ObjectNumber},
};
use uuid::Uuid;

/// stage of a tournament
// ToDo: remove allow(dead_code) flag
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stage {
    /// id and version of stage in tournament
    id_version: IdVersion,
    /// id of tournament
    tournament_id: Uuid,
    /// scheduled stage number in tournament
    number: u32,
}

impl ObjectIdVersion for Stage {
    fn get_id_version(&self) -> IdVersion {
        self.id_version
    }
}

impl ObjectNumber for Stage {
    fn get_object_number(&self) -> u32 {
        self.number
    }
}

impl Stage {
    /// Returns the tournament ID.
    pub fn get_tournament_id(&self) -> Uuid {
        self.tournament_id
    }
}
