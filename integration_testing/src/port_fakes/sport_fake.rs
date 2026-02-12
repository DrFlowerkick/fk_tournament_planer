//! sport port fake and testing of SportPluginManagerMap

use app_core::{
    EntrantGroupScore, Match, SportConfig, SportPort, SportResult,
    utils::{id_version::IdVersion, traits::ObjectIdVersion, validation::ValidationErrors},
};
use leptos::prelude::*;
use serde_json::Value;
use shared::SportPortWebUi;
use std::time::Duration;
use uuid::Uuid;

// A mock sport plugin for testing purposes.
pub struct MockSport {
    pub id: Uuid,
    pub name: &'static str,
}

impl ObjectIdVersion for MockSport {
    fn get_id_version(&self) -> IdVersion {
        IdVersion::new(self.id(), Some(0))
    }
}

impl MockSport {
    pub fn id(&self) -> Uuid {
        self.id
    }
}
impl SportPortWebUi for MockSport {
    fn render_plugin_selection(&self) -> AnyView {
        view! { <div>{format!("Select {}", self.name)}</div> }.into_any()
    }
    fn render_preview(&self, _config: &SportConfig) -> AnyView {
        view! { <div>{"Mock Preview"}</div> }.into_any()
    }
    fn render_detailed_preview(&self, config: &SportConfig) -> AnyView {
        view! { <div>{format!("Extended Mock Preview of: {}", config.get_name())}</div> }.into_any()
    }
    fn render_configuration(&self) -> AnyView {
        view! { <div>{"Configuration UI"}</div> }.into_any()
    }
}

impl SportPort for MockSport {
    fn name(&self) -> &'static str {
        self.name
    }

    fn get_default_config(&self) -> Value {
        serde_json::json!({})
    }

    fn validate_config_values(
        &self,
        _config: &SportConfig,
        _err: ValidationErrors,
    ) -> SportResult<()> {
        Ok(())
    }

    fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> {
        Ok(Duration::from_secs(0))
    }

    fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> {
        Ok(())
    }

    fn get_entrant_group_score(
        &self,
        _config: &SportConfig,
        group_id: Uuid,
        entrant_id: Uuid,
        _all_matches: &[Match],
    ) -> SportResult<EntrantGroupScore> {
        Ok(EntrantGroupScore {
            entrant_id,
            group_id,
            victory_points: 0.0,
            relative_score: 0,
            total_score: 0,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use app_core::{SportPluginManagerPort, utils::traits::ObjectIdVersion};
    use sport_plugin_manager::SportPluginManagerMap;
    use std::sync::Arc;

    #[test]
    fn test_register_and_get() {
        let mut manager = SportPluginManagerMap::new();
        let sport_id = Uuid::new_v4();
        let plugin = Arc::new(MockSport {
            id: sport_id,
            name: "TestSport",
        });

        manager.register(plugin.clone()).unwrap();

        let found = manager.get(&sport_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_id_version().get_id(), sport_id);
    }

    #[test]
    fn test_get_not_found() {
        let manager = SportPluginManagerMap::new();
        let sport_id = Uuid::new_v4();
        let found = manager.get(&sport_id);
        assert!(found.is_none());
    }

    #[test]
    fn test_list_plugins() {
        let mut manager = SportPluginManagerMap::new();
        let sport_id1 = Uuid::new_v4();
        let sport_id2 = Uuid::new_v4();
        let plugin1 = Arc::new(MockSport {
            id: sport_id1,
            name: "TestSport1",
        });
        let plugin2 = Arc::new(MockSport {
            id: sport_id2,
            name: "TestSport2",
        });

        manager.register(plugin1).unwrap();
        manager.register(plugin2).unwrap();

        let list = manager.list();
        assert_eq!(list.len(), 2);
        assert!(
            list.iter()
                .any(|p| p.get_id_version().get_id() == sport_id1)
        );
        assert!(
            list.iter()
                .any(|p| p.get_id_version().get_id() == sport_id2)
        );
    }

    #[test]
    fn test_overwrite_plugin_should_fail() {
        let mut manager = SportPluginManagerMap::new();
        let sport_id = Uuid::new_v4();
        let plugin1 = Arc::new(MockSport {
            id: sport_id,
            name: "OldSport",
        });
        let plugin2 = Arc::new(MockSport {
            id: sport_id,
            name: "NewSport",
        });

        manager.register(plugin1).unwrap();
        assert_eq!(manager.get(&sport_id).unwrap().name(), "OldSport");

        assert!(manager.register(plugin2).is_err());
    }
}
