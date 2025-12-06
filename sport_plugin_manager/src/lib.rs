//! Implementation of sport plugin manager port

use anyhow::{Context, Result, bail};
use app_core::{SportPluginManagerPort, SportPort};
use shared::SportConfigPreview;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// A concrete implementation of the `SportPluginManagerPort` (see mod server) that uses a `HashMap`
/// to store and retrieve sport plugins.
#[derive(Clone, Default)]
pub struct SportPluginManagerMap {
    plugins: HashMap<Uuid, Arc<dyn SportConfigPreview>>,
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
    /// If a plugin with the same ID already exists, an error is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use sport_plugin_manager::SportPluginManagerMap;
    /// # use app_core::{SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore, utils::id_version::{IdVersion, VersionId}};
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::SportConfigPreview;
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl VersionId for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, 0)
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportConfigPreview for MockSport {
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// let sport_id = Uuid::new_v4();
    /// let plugin = Arc::new(MockSport { id: sport_id, name: "MockSport" });
    ///
    /// manager.register(plugin).unwrap();
    ///
    /// assert!(manager.get(&sport_id).is_some());
    /// ```
    pub fn register(&mut self, plugin: Arc<dyn SportConfigPreview>) -> Result<()> {
        let plugin_id = plugin
            .get_id_version()
            .get_id()
            .context("Sport Plugin must provide id.")?;
        if self.plugins.contains_key(&plugin_id) {
            bail!("A plugin with ID {} is already registered", plugin_id);
        }
        if self.plugins.values().any(|p| p.name() == plugin.name()) {
            bail!(
                "A plugin with name '{}' is already registered",
                plugin.name()
            );
        }
        self.plugins.insert(plugin_id, plugin);
        Ok(())
    }
    /// Retrieves a registered sport configuration preview plugin by its unique ID.
    ///
    /// Returns `None` if no plugin with the given ID is found.
    ///
    /// # Example
    ///
    /// ```
    /// # use sport_plugin_manager::SportPluginManagerMap;
    /// # use app_core::{SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore, utils::id_version::{IdVersion, VersionId}};
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::SportConfigPreview;
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl VersionId for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, 0)
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportConfigPreview for MockSport {
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// let sport_id = Uuid::new_v4();
    /// let plugin = Arc::new(MockSport { id: sport_id, name: "MockSport" });
    /// manager.register(plugin).unwrap();
    ///
    /// // Get an existing plugin
    /// let found_plugin = manager.get_preview(&sport_id);
    /// assert!(found_plugin.is_some());
    /// assert_eq!(found_plugin.unwrap().get_id_version().get_id().unwrap(), sport_id);
    ///
    /// // Try to get a non-existent plugin
    /// let not_found_plugin = manager.get_preview(&Uuid::new_v4());
    /// assert!(not_found_plugin.is_none());
    /// ```
    pub fn get_preview(&self, sport_id: &Uuid) -> Option<Arc<dyn SportConfigPreview>> {
        self.plugins.get(sport_id).cloned()
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
    /// # use app_core::{SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore, utils::id_version::{IdVersion, VersionId}};
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::SportConfigPreview;
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl VersionId for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, 0)
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportConfigPreview for MockSport {
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// let sport_id = Uuid::new_v4();
    /// let plugin = Arc::new(MockSport { id: sport_id, name: "MockSport" });
    /// manager.register(plugin).unwrap();
    ///
    /// // Get an existing plugin
    /// let found_plugin = manager.get(&sport_id);
    /// assert!(found_plugin.is_some());
    /// assert_eq!(found_plugin.unwrap().get_id_version().get_id().unwrap(), sport_id);
    ///
    /// // Try to get a non-existent plugin
    /// let not_found_plugin = manager.get(&Uuid::new_v4());
    /// assert!(not_found_plugin.is_none());
    /// ```
    fn get(&self, sport_id: &Uuid) -> Option<Arc<dyn SportPort>> {
        self.plugins
            .get(sport_id)
            .map(|p| p.clone() as Arc<dyn SportPort>)
    }

    /// Returns a list of all registered sport plugins.
    ///
    /// The order of the plugins in the returned `Vec` is not guaranteed.
    ///
    /// # Example
    ///
    /// ```
    /// # use sport_plugin_manager::SportPluginManagerMap;
    /// # use app_core::{SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore, utils::id_version::{IdVersion, VersionId}};
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::SportConfigPreview;
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl VersionId for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, 0)
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportConfigPreview for MockSport {
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// manager.register(Arc::new(MockSport { id: Uuid::new_v4(), name: "Sport1" })).unwrap();
    /// manager.register(Arc::new(MockSport { id: Uuid::new_v4(), name: "Sport2" })).unwrap();
    ///
    /// let all_plugins = manager.list();
    /// assert_eq!(all_plugins.len(), 2);
    /// ```
    fn list(&self) -> Vec<Arc<dyn SportPort>> {
        self.plugins
            .values()
            .map(|p| p.clone() as Arc<dyn SportPort>)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_core::{
        EntrantGroupScore, Match, SportConfig, SportResult,
        utils::id_version::{IdVersion, VersionId},
    };
    use leptos::prelude::*;
    use serde_json::Value;
    use std::time::Duration;

    // A mock sport plugin for testing purposes.
    struct MockSport {
        id: Uuid,
        name: &'static str,
    }

    impl VersionId for MockSport {
        fn get_id_version(&self) -> IdVersion {
            IdVersion::new(self.id(), 0)
        }
    }

    impl MockSport {
        fn id(&self) -> Uuid {
            self.id
        }
    }
    impl SportConfigPreview for MockSport {
        fn render_preview(&self, _config: &SportConfig) -> AnyView {
            view! { <div>{"Mock Preview"}</div> }.into_any()
        }
    }

    impl SportPort for MockSport {
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

        manager.register(plugin.clone()).unwrap();

        let found = manager.get(&sport_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_id_version().get_id(), Some(sport_id));
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
                .any(|p| p.get_id_version().get_id() == Some(sport_id1))
        );
        assert!(
            list.iter()
                .any(|p| p.get_id_version().get_id() == Some(sport_id2))
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
