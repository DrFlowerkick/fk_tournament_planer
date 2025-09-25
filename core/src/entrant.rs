// entrants of tournament

use uuid::Uuid;

/// entrant of tournament; either team or individual athlete
#[derive(Debug, Clone)]
pub struct Entrant {
    /// id of entrant in tournament
    id: Uuid,
    /// optional global id of entrant, if entrant stats are kept in database
    global_id: Option<Uuid>,
    /// name of entrant
    name: String,
    // ToDo: more parameters for scoring including an optional "score mate", who is responsible for providing scores of entrant
}

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