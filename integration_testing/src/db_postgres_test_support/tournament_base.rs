use app_core::{
    TournamentBase, TournamentMode, TournamentState, TournamentType, utils::id_version::IdVersion,
};
use uuid::Uuid;

/// Build a valid "new" TournamentBase with deterministic fields.
pub fn make_new_tournament_base(label: &str, sport_id: Uuid) -> TournamentBase {
    let mut tb = TournamentBase::new(IdVersion::New);
    tb.set_name(format!("Tournament {label}"))
        .set_sport_id(sport_id)
        .set_num_entrants(16)
        .set_tournament_type(TournamentType::Scheduled)
        .set_tournament_mode(TournamentMode::SingleStage)
        .set_tournament_state(TournamentState::Scheduling);
    tb
}

/// Mutate the tournament base to a second version (change some fields).
pub fn mutate_tournament_base_v2(mut tb: TournamentBase) -> TournamentBase {
    tb.set_name("Updated Tournament V2")
        .set_num_entrants(32)
        .set_tournament_state(TournamentState::ActiveStage(0));
    tb
}

/// A second mutation variant to differentiate two competing updates.
pub fn mutate_tournament_base_v3(mut tb: TournamentBase) -> TournamentBase {
    tb.set_name("Updated Tournament V3")
        .set_num_entrants(8)
        .set_tournament_state(TournamentState::Finished);
    tb
}

/// Compare only the fields we change in mutations to decide the "winner" semantics.
pub fn same_semantics(a: &TournamentBase, b: &TournamentBase) -> bool {
    a.get_name() == b.get_name()
        && a.get_num_entrants() == b.get_num_entrants()
        && a.get_tournament_state() == b.get_tournament_state()
}
