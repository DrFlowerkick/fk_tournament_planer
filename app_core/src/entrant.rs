// entrants of tournament

use uuid::Uuid;

/// entrant of tournament; either team or individual athlete
#[derive(Debug, Clone)]
pub struct Entrant {
    /// id of entrant in tournament
    pub id: Uuid,
    /// id of tournament
    tournament_id: Uuid,
    /// optional global id of entrant, if entrant stats are kept in database
    global_id: Option<Uuid>,
    /// name of entrant
    name: String,
    /// members of entrant
    members: Vec<Member>,
}

// ToDo: move this into generic people mod?
/// member of entrant, if entrant is team
#[derive(Debug, Clone)]
pub struct Member {
    /// id of member in tournament
    id: Uuid,
    /// optional global id of member, if member stats are kept in database
    global_id: Option<Uuid>,
    /// name of member
    name: String,
}
