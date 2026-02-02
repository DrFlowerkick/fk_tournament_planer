use app_core::SportConfig;
use serde_json::json;
use uuid::Uuid;

/// Build a valid "new" SportConfig with deterministic fields.
pub fn make_new_sport_config(label: &str, sport_id: Uuid) -> SportConfig {
    let mut sc = SportConfig::default();
    sc.set_sport_id(sport_id)
        .set_name(format!("SportConfig {label}"))
        .set_config(json!({ "rules": "default", "players": 11, "label": label }));
    sc
}

/// Mutate the sport config to a second version (change some fields).
pub fn mutate_sport_config_v2(mut sc: SportConfig) -> SportConfig {
    sc.set_name("Updated SportConfig V2")
        .set_config(json!({ "rules": "v2", "players": 5 }));
    sc
}

/// A second mutation variant to differentiate two competing updates.
pub fn mutate_sport_config_v3(mut sc: SportConfig) -> SportConfig {
    sc.set_name("Updated SportConfig V3")
        .set_config(json!({ "rules": "v3", "players": 7 }));
    sc
}

/// Compare only the fields we change in mutations to decide the "winner" semantics.
pub fn same_semantics(a: &SportConfig, b: &SportConfig) -> bool {
    a.get_name() == b.get_name() && a.get_config() == b.get_config()
}
