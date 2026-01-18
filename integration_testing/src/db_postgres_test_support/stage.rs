use app_core::{Stage, utils::id_version::IdVersion};
use uuid::Uuid;

/// Build a valid "new" Stage.
pub fn make_new_stage(t_id: Uuid, number: u32) -> Stage {
    let mut s = Stage::new(IdVersion::New);
    s.set_tournament_id(t_id)
        .set_number(number)
        .set_num_groups(2); // Default used in tests
    s
}

/// Mutate the stage to a second version.
pub fn mutate_stage_v2(mut s: Stage) -> Stage {
    s.set_num_groups(4);
    s
}

/// A second mutation variant.
pub fn mutate_stage_v3(mut s: Stage) -> Stage {
    s.set_num_groups(8);
    s
}

/// Compare fields relevant for semantics check
pub fn same_semantics(a: &Stage, b: &Stage) -> bool {
    a.get_tournament_id() == b.get_tournament_id()
        && a.get_number() == b.get_number()
        && a.get_num_groups() == b.get_num_groups()
}
