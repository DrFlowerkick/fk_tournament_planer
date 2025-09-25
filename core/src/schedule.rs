// scheduling a tournament

use uuid::Uuid;
use time::OffsetDateTime;
use crate::ScoringPolicy;

/// schedule of tournament
#[derive(Debug, Clone)]
pub struct Schedule {
    /// id of schedule
    id: Uuid,
    /// date of (first) day of tournament
    date: OffsetDateTime,
    /// number of stations
    /// station represents all kinds of sport areas to carry out matches, e.g. courts, tables, fields
    num_stations: u16,
    /// stages of tournament
    stages: Vec<ScheduledStage>,
    // ToDo: add further parameters required to schedule a tournament
}


/// one stage of a tournament
#[derive(Debug, Clone)]
pub struct ScheduledStage {
    /// id of stage in tournament
    id: Uuid,
    /// id of scheduled Stage
    schedule_id: Uuid,
    /// id of tournament
    tournament_id: Uuid,
    /// groups of stage
    groups: Vec<ScheduledGroup>,
}

#[derive(Debug, Clone)]
pub enum Mode {
    RoundRobin,
    KO,
    KOFullPlayOut,
    Swiss,
}

/// scheduled group of a stage
#[derive(Debug, Clone)]
pub struct ScheduledGroup {
    /// id of group in tournament
    id: Uuid,
    /// id of scheduled group
    schedule_id: Uuid,
    /// id of stage
    stage_id: Uuid,
    /// match making mode of group
    /// Normally all groups of one stage share the same mode. This may be not true for final stage,
    /// if number of entrants forces groups with different number of entrants. If you have 9
    /// entrants and wan to have half finals and finals, than a group of top for entrants is required,
    /// playing out the tournament in finals in KO Play Out mode.
    /// The remaining 5 entrants may share a second group or may even divided in two groups with 3
    /// respectively 2 entrants. In either case, KO Play Out is not suited for these group sizes,
    /// therefore another mode like round robin is required.
    mode: Mode,
    /// scoring policy of group
    /// Normally all groups of one stage share the same scoring policy. This may be not true for final stage,
    /// as it is for match making mode for the same reasons. If there is one group with only two entrants,
    /// you may wish to increase number of sets to win the match (if applicable), so that both teams
    /// have more time to play.
    scoring_policy: ScoringPolicy,
    /// entrants of this group
    entrants: Vec<ScheduledEntrant>,
    /// rounds of matches of this group
    rounds: Vec<ScheduledRound>,
    /// current active round; ToDo: do we need this?
    active_round: usize,
}

/// round of scheduled matches
#[derive(Debug, Clone)]
pub struct ScheduledRound {
    /// id of round in tournament
    id: Uuid,
    /// matches of round
    matches: Vec<ScheduledMatch>,
    /// if odd number of entrants, one entrant must pause this round
    /// index of entrant in entrant list of group sorted by rank
    pause: Option<usize>,
}

/// scheduled match of tournament
#[derive(Debug, Clone)]
pub struct ScheduledMatch {
    /// id of match in tournament
    id: Uuid,
    /// id of scheduled round
    round_id: Uuid,
    /// scheduled entrant a
    side_a: ScheduledEntrant,
    /// scheduled entrant b
    side_b: ScheduledEntrant,
    // ToDo: schedule information: time, date, court
}

/// scheduled entrant for match
#[derive(Debug, Clone)]
pub enum ScheduledEntrant {
    /// Entrant referenced by id; used for first stage
    /// Uuid: id of entrant
    Entrant(Uuid),
    /// rank of entrant after concluded stage
    /// Uuid of stage, usize: index of entrant in entrant list sorted by stage rank
    StageRank(Uuid, usize),
    /// rank of entrant in group after concluded stage
    /// Uuid of stage, usize: index of entrant in entrant list of group sorted by group rank
    GroupRank(Uuid, usize),
    /// In Swiss system entrants are allocated to matches during tournament depending on
    /// their achieved results and the results of their opponents.
    // ToDo: In pure Swiss system, index of ranks may be precalculated. But it may be
    // useful to use for up to first n rounds some random entrant matching to prevent
    // best teams from meeting to early in tournament. If this option is not used,
    // we could add here precalculated index of rank.
    Swiss,
}