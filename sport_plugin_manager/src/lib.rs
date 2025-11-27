//! Implementation of sport plugin manager port

use app_core::{SportPluginManagerPort, SportPort};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// A concrete implementation of the `SportPluginManagerPort` that uses a `HashMap`
/// to store and retrieve sport plugins.
#[derive(Clone, Default)]
pub struct SportPluginManagerMap {
    plugins: HashMap<Uuid, Arc<dyn SportPort>>,
}

impl SportPluginManagerMap {
    /// Creates a new, empty plugin manager.
    ///
    /// # Example
    ///
    /// ```
    /// # use sport_plugin_manager::SportPluginManagerMap;
    /// let manager = SportPluginManagerMap::new();
    /// // The manager is now ready to register plugins.
    /// ```
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Registers a new sport plugin with the manager.
    ///
    /// If a plugin with the same ID already exists, it will be overwritten.
    ///
    /// # Example
    ///
    /// ```
    /// # use sport_plugin_manager::SportPluginManagerMap;
    /// # use app_core::{SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore};
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use serde_json::Value;
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl SportPort for MockSport {
    /// #     fn id(&self) -> Uuid { self.id }
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// let sport_id = Uuid::new_v4();
    /// let plugin = Arc::new(MockSport { id: sport_id, name: "MockSport" });
    ///
    /// manager.register(plugin);
    ///
    /// assert!(manager.get(&sport_id).is_some());
    /// ```
    pub fn register(&mut self, plugin: Arc<dyn SportPort>) {
        self.plugins.insert(plugin.id(), plugin);
    }
}

impl SportPluginManagerPort for SportPluginManagerMap {
    /// Retrieves a registered sport plugin by its unique ID.
    ///
    /// Returns `None` if no plugin with the given ID is found.
    ///
    /// # Example
    ///
    /// ```
    /// # use sport_plugin_manager::SportPluginManagerMap;
    /// # use app_core::{SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore};
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use serde_json::Value;
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl SportPort for MockSport {
    /// #     fn id(&self) -> Uuid { self.id }
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// let sport_id = Uuid::new_v4();
    /// let plugin = Arc::new(MockSport { id: sport_id, name: "MockSport" });
    /// manager.register(plugin);
    ///
    /// // Get an existing plugin
    /// let found_plugin = manager.get(&sport_id);
    /// assert!(found_plugin.is_some());
    /// assert_eq!(found_plugin.unwrap().id(), sport_id);
    ///
    /// // Try to get a non-existent plugin
    /// let not_found_plugin = manager.get(&Uuid::new_v4());
    /// assert!(not_found_plugin.is_none());
    /// ```
    fn get(&self, sport_id: &Uuid) -> Option<Arc<dyn SportPort>> {
        self.plugins.get(sport_id).cloned()
    }

    /// Returns a list of all registered sport plugins.
    ///
    /// The order of the plugins in the returned `Vec` is not guaranteed.
    ///
    /// # Example
    ///
    /// ```
    /// # use sport_plugin_manager::SportPluginManagerMap;
    /// # use app_core::{SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore};
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use serde_json::Value;
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl SportPort for MockSport {
    /// #     fn id(&self) -> Uuid { self.id }
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// manager.register(Arc::new(MockSport { id: Uuid::new_v4(), name: "Sport1" }));
    /// manager.register(Arc::new(MockSport { id: Uuid::new_v4(), name: "Sport2" }));
    ///
    /// let all_plugins = manager.list();
    /// assert_eq!(all_plugins.len(), 2);
    /// ```
    fn list(&self) -> Vec<Arc<dyn SportPort>> {
        self.plugins.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::{EntrantGroupScore, Match, SportConfig, SportResult};
    use serde_json::Value;
    use std::time::Duration;

    // A mock sport plugin for testing purposes.
    struct MockSport {
        id: Uuid,
        name: &'static str,
    }

    impl SportPort for MockSport {
        fn id(&self) -> Uuid {
            self.id
        }

        fn name(&self) -> &'static str {
            self.name
        }

        fn get_default_config(&self) -> Value {
            serde_json::json!({})
        }

        fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> {
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

    #[test]
    fn test_register_and_get() {
        let mut manager = SportPluginManagerMap::new();
        let sport_id = Uuid::new_v4();
        let plugin = Arc::new(MockSport {
            id: sport_id,
            name: "TestSport",
        });

        manager.register(plugin.clone());

        let found = manager.get(&sport_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), sport_id);
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

        manager.register(plugin1);
        manager.register(plugin2);

        let list = manager.list();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|p| p.id() == sport_id1));
        assert!(list.iter().any(|p| p.id() == sport_id2));
    }

    #[test]
    fn test_overwrite_plugin() {
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

        manager.register(plugin1);
        assert_eq!(manager.get(&sport_id).unwrap().name(), "OldSport");

        manager.register(plugin2);
        assert_eq!(manager.get(&sport_id).unwrap().name(), "NewSport");
        assert_eq!(manager.list().len(), 1);
    }
}
