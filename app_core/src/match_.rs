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
    /// tournament id
    tournament_id: Uuid,
    /// id of sport
    sport_id: Uuid,
    /// stage id
    stage_id: Uuid,
    /// id of group
    group_id: Uuid,
    /// id of round
    round_id: Uuid,
    /// number of match in round
    number: u32,
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

impl Match {
    /// Returns the match ID.
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
    /// Returns the tournament ID.
    pub fn get_tournament_id(&self) -> &Uuid {
        &self.tournament_id
    }
    /// Returns the sport ID.
    pub fn get_sport_id(&self) -> &Uuid {
        &self.sport_id
    }
    /// Returns the stage ID.
    pub fn get_stage_id(&self) -> &Uuid {
        &self.stage_id
    }
    /// Returns the group ID.
    pub fn get_group_id(&self) -> &Uuid {
        &self.group_id
    }
    /// Returns the round ID.
    pub fn get_round_id(&self) -> &Uuid {
        &self.round_id
    }
    /// Returns the entrant IDs of both sides if they are concrete entrants.
    pub fn get_entrants(&self) -> Option<(&Uuid, &Uuid)> {
        match (&self.side_a, &self.side_b) {
            (ScheduledEntrant::Entrant(id_a), ScheduledEntrant::Entrant(id_b)) => {
                Some((id_a, id_b))
            }
            _ => None,
        }
    }
    /// Returns if match has been played, i.e., if scores are available.
    pub fn is_played(&self) -> bool {
        !self.score_a.is_empty() && !self.score_b.is_empty()
    }
    /// Returns the scores of both entrants as references to their respective vectors.
    pub fn get_scores(&self) -> (&Vec<u16>, &Vec<u16>) {
        (&self.score_a, &self.score_b)
    }
    /// Creates a new match with scores (played match).
    /// Useful for testing and initializing played matches.
    // ToDo: try later to find a better way to create played matches for testing
    pub fn new_played(
        id: Uuid,
        entrant_a: Uuid,
        entrant_b: Uuid,
        sport_id: Uuid,
        score_a: Vec<u16>,
        score_b: Vec<u16>,
    ) -> Self {
        Self {
            id,
            tournament_id: Uuid::nil(),
            sport_id,
            stage_id: Uuid::nil(),
            group_id: Uuid::nil(),
            round_id: Uuid::nil(),
            number: 0,
            side_a: ScheduledEntrant::Entrant(entrant_a),
            side_b: ScheduledEntrant::Entrant(entrant_b),
            station: 0,
            start_at: Local::now(),
            score_a,
            score_b,
        }
    }
}
