// orchestration of tournament

use crate::EntrantGroupScore;
use uuid::Uuid;

/// orchestration of tournament
#[derive(Debug, Clone)]
pub struct Orchestration {
    /// id of orchestration
    id: Uuid,
    /// stages of tournament
    stages: Vec<Stage>,
    /// state of tournament
    state: OrchestrationState,
}

/// state of orchestration
#[derive(Debug, Clone)]
pub enum OrchestrationState {
    Pending,
    ActiveStage(Uuid),
    Finished,
}

/// stage of a tournament
#[derive(Debug, Clone)]
pub struct Stage {
    /// id of stage in tournament
    id: Uuid,
    /// id of scheduled Stage
    schedule_id: Uuid,
    /// id of tournament
    tournament_id: Uuid,
    /// groups of stage
    groups: Vec<Group>,
}

/// group of a stage
#[derive(Debug, Clone)]
pub struct Group {
    /// id of group in tournament
    id: Uuid,
    /// id of scheduled group
    schedule_id: Uuid,
    /// id of stage
    stage_id: Uuid,
    /// entrants of this group
    entrants: Vec<Uuid>,
    /// scoring of entrants in this group
    entrant_scores: Vec<EntrantGroupScore>,
    /// rounds of matches of this group
    rounds: Vec<Round>,
    /// current active round; ToDo: do we need this?
    active_round: usize,
}

/// round of matches
#[derive(Debug, Clone)]
pub struct Round {
    /// id of round in tournament
    id: Uuid,
    /// id of scheduled round
    schedule_id: Uuid,
    /// id of group
    group_id: Uuid,
    /// matches of round
    matches: Vec<Match>,
    /// if odd number of entrants, one entrant must pause this round
    /// Uuid of Entrant
    pause: Option<Uuid>,
}

/// match of tournament
#[derive(Debug, Clone)]
pub struct Match {
    /// id of match in tournament
    id: Uuid,
    /// id of scheduled match
    schedule_id: Uuid,
    /// if if round
    round_id: Uuid,
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