// group of a stage

use uuid::Uuid;

/// group of a stage
#[derive(Debug, Clone)]
pub struct Group {
    /// id of group in tournament
    id: Uuid,
    /// id of stage
    stage_id: Uuid,
    /// group number
    number: usize,
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
    scoring_policy: Uuid,
    /// timing structure of a match of this group
    /// Normally all groups of one stage share the same timing structure. This may be not true for final stage,
    /// as it is for match making mode for the same reasons. If there is one group with only two entrants,
    /// you may wish to increase number of sets to win the match (if applicable), therefore changing
    /// the timing structure for this group.
    timing: Uuid,
    /// scheduled entrants of this group
    // ToDo: do we need a list of entrants in group?
    scheduled_entrants: Vec<ScheduledEntrant>,
    /// scoring of entrants in this group, referenced by id
    entrant_scores: Vec<Uuid>,
    /// rounds of matches of this group, referenced by id, sorted by round number
    rounds: Vec<Uuid>,
}



#[derive(Debug, Clone)]
pub enum Mode {
    RoundRobin,
    KOFullPlayOut,
    KO,
    Swiss,
}

/// scheduled entrant for a match
#[derive(Debug, Clone)]
pub enum ScheduledEntrant {
    /// Entrant referenced by id; used for first stage and stages, which previous stages are done
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
