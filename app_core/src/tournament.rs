/// Tournament functionality
///
/// This tournament app aims at sports, where teams or individual athletes compete
/// with each other in direct matches, which can either be won or lost or may
/// end in a draw (depends upon the respective sport). In context of this app
/// these competitors are called entrants of the tournament.
///
/// A tournament consists of one or more stages.
///
/// In each stage entrants are grouped in one or more groups. In first stage,
/// group mapping of entrants may be done by rank (e.g. world rank), equal
/// distributed rank (a.k.a. "counting trough": if you have 20 entrants and
/// 5 groups, you count trough from top rank to lowest rank from 1 to 5,
/// in which the number represents the mapped group), or random. When moving
/// to next stage, current stage rank decides group mapping (see below).
/// Snake muster anschauen. Eher der Standard international.
/// ToDo: die number of courts können über das Turnier ggf. variieren.
/// Knock Out mit Wild Card? Recherchieren...
/// Ein Turnierort sollte optional sein, für quick and dirty tournaments
/// Dafür sollten dann auh einfach dummy entrants möglich sein, die einfach per Nummer durchgezählt sind.
///
/// I each group all entrants have matches against each other after a
/// certain mode. In final stage this is usually KO Play Out (see below), while
/// earlier stages use round robin. The mode depends upon group size. KO Play Out
/// requires a group size of 2^n with n= 1, 2, 3, ... entrants. These matches are
/// organized in rounds. With an even number of entrants in group, each entrant
/// has one match per round (if not dropped out of tournament, see KO below).
/// An odd number of entrants implies a pause for one entrant in each round.
/// ToDo: Zuordnung zu de stations: die top gesetzten Teams werden festen Stationen zugeteilt,
/// die anderen müssen wandern. StationPolicy?
///
/// After all rounds of matches in all groups of current stage are done, a stage
/// ranking is generated for all entrants. This ranking depends upon ranking of
/// previous stage, if any, and results of current stage. Ranking of previous stage
/// is represented by group number of current stage. Depending upon stage ranking
/// the entrants are mapped to groups of next stage: if you have 20 entrants
/// and 5 groups in next stage, the top 4 are mapped to group 1, next top 4
/// to group 2, etc.. Final ranking of tournament is equal to ranking of
/// final stage.
///
/// Tournaments with [Swiss system](https://en.wikipedia.org/wiki/Swiss-system_tournament)
/// work different. Swiss system is similar to round robin, but instead of a full
/// round robin each entrant has a certain number of matches less than (number of entrants - 1).
/// After each round of matches ranking is resolved. Match partners of next round
/// depend upon current ranking. The core idea is, that current best entrant has his next
/// match against the next best entrant, which it has not faced yet. Same is done for
/// the remaining entrants until all entrants have a match partner in next round.
/// In case of odd number of entrants, one entrant gains a "free win" and pauses
/// for one round. Each entrant may have only one free win. After certain number of
/// rounds the ranking of entrants is stable enough to represent the final ranking.
/// The recommended number of rounds to play is 'log_2(number of entrants) + 2' or more.
/// The maximum number of rounds is equal to round robin.
/// Hier ggf. mit Buffern arbeiten. Nochmal recherchieren.
/// Double elimination wird durchaus verwendet (z.B. Free Style)
/// Ring System. Man stellt die Mannschaften in einem Ring auf und spielt gegen die Nachbarn
/// -> Recherchieren
///
/// Harte Time caps im Timing ergänzen
/// Noch nicht gestartete stages sollten auch bei gestarteten Turnier nur bearbeitbar im schedule sein.
/// Tie Breaker sollen durch den turnierdirektor konfigurierbar sein.
///
/// The Swiss system can be integrated in the generic tournament structure by using a stage
/// for each round of Swiss matches. Therefore a Swiss stage contains only one
/// group (all entrants) and one round of matches.
///
/// Depending upon tournament style, some entrants may drop out of tournament
/// after each stage or during KO mode. KO mode is normally used in final stage,
/// if at all. KO vs KO Play Out: in KO the loser drops out of tournament,
/// while in KO Play Out the losers match against each other to play out
/// lower ranking.
///
/// Ranking is resolved usually by comparing wins, losses, and draws, if applicable.
/// Normally wins and draws give some amount of victory points (e.g. 1 for wins and
/// 0.5 for draws). Most victory points result in best rank. Ranks start from 1 (best
/// rank) to n (last rank) with n being the number of entrants. In all stages after the first,
/// ranks are first resolved in each group and than ordered by group number:
/// if you have 20 entrants and 5 groups, rank 1 to 4 of first group have rank 1 to 4
/// of all entrants, rank 1 to 4 of second group have rank 5 to 8 of all entrants, etc..
///
/// There are several ways to break ties, e.g.
/// - delta points
/// - total points
/// - direct comparison (did entrants face each other? If yes, who won?)
/// - initial tournament ranking (e.g. inferred from world ranking system)
/// - coin flip
///
/// Swiss mode uses [buchholz score](https://en.wikipedia.org/wiki/Buchholz_system)
/// or something similar as primary way of breaking ties. These (and more) options can be
/// combined.
///
///
/// data structures
///
/// One option may be to put all parameters in one big struct. Since this will get more
/// and more confusing with growing size of struct, I suggest separate structs for the
/// components of the tournament, which data will be persisted via  database.
///
use crate::{Core, PostalAddress, ServerContext};
use anyhow::Result;
use chrono::{DateTime, Local};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tournament {
    /// id of tournament
    id: Uuid,
    /// name of tournament
    name: String,
    /// location of tournament
    location: Uuid,
    /// date and timing of tournament days, referenced by id, sorted by day number
    tournament_days: Vec<Uuid>,
    /// number of stations
    /// station represents all kinds of sport areas to carry out matches, e.g. courts, tables, fields
    num_stations: u16,
    /// tie breaker policy
    tie_breaker_policy: Uuid,
    /// entrants of tournament
    entrants: HashSet<Uuid>,
    /// stages of tournament, referenced by id, sorted by stage number
    stages: Vec<Uuid>,
    /// state of tournament
    status: Status,
}

/// activity of orchestration
#[derive(Debug, Clone)]
pub enum Status {
    Pending,
    ActiveStage(Uuid),
    Finished,
}

pub struct TournamentState {
    tournament: Tournament,
}

impl<C: Clone> Core<TournamentState, C> {
    pub fn is_valid(&self) -> bool {
        todo!()
    }
}

impl Core<TournamentState, ServerContext> {
    pub async fn tournament_action(
        &mut self,
        action: TournamentActions,
    ) -> Result<TournamentViewModel> {
        match action {
            TournamentActions::ChangeName(name) => self.state.tournament.name = name,
            TournamentActions::ChangeLocation(location) => {
                self.state.tournament.location = location
            }
            TournamentActions::AddEntrant(entrant) => {
                self.state.tournament.entrants.insert(entrant);
            }
            TournamentActions::RemoveEntrant(entrant) => {
                self.state.tournament.entrants.remove(&entrant);
            }
        }
        self.render_view_model().await
    }
    pub async fn render_view_model(&self) -> Result<TournamentViewModel> {
        let location = self
            .as_postal_address_state()
            .load(self.state.tournament.location)
            .await?
            .cloned();
        let start_at = if let Some(day_timing_id) = self.state.tournament.tournament_days.get(0)
            && let Some(day_timing) = self.get_tournament_day_timing_state(*day_timing_id)?
        {
            Some(day_timing.get().date)
        } else {
            None
        };
        Ok(TournamentViewModel {
            name: self.state.tournament.name.clone(),
            location,
            start_at,
            num_entrants: self.state.tournament.entrants.len(),
            valid_schedule: self.is_valid(),
        })
    }
}

pub enum TournamentActions {
    ChangeName(String),
    ChangeLocation(Uuid),
    AddEntrant(Uuid),
    RemoveEntrant(Uuid),
}

pub struct TournamentViewModel {
    pub name: String,
    pub location: Option<PostalAddress>,
    pub start_at: Option<DateTime<Local>>,
    pub num_entrants: usize,
    pub valid_schedule: bool,
}

pub struct NewTournamentState {}
