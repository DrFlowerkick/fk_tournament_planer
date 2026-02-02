//! Implementation of sport plugin manager port

use anyhow::{Result, bail};
use app_core::{SportPluginManagerPort, SportPort};
use shared::SportPortWebUi;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// A concrete implementation of the `SportPluginManagerPort` (see mod server) that uses a `HashMap`
/// to store and retrieve sport plugins.
#[derive(Clone, Default)]
pub struct SportPluginManagerMap {
    plugins: HashMap<Uuid, Arc<dyn SportPortWebUi>>,
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
    /// # use app_core::{
    /// #   SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore,
    /// #   utils::{
    /// #       id_version::IdVersion,
    /// #       traits::ObjectIdVersion,
    /// #       validation::ValidationErrors,
    /// #   }
    /// # };
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::{SportPortWebUi, RenderCfgProps};
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl ObjectIdVersion for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, Some(0))
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig, _err: ValidationErrors) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportPortWebUi for MockSport {
    /// #     fn render_plugin_selection(&self) -> AnyView {
    /// #         view! { <div>{ "Mock Plugin Selection" }</div> }.into_any()
    /// #     }
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// #     fn render_dropdown(&self, config: &SportConfig) -> AnyView {
    /// #         view! { <div>{format!("Dropdown: {}", config.get_name())}</div> }.into_any()
    /// #     }
    /// #     fn render_configuration(&self, _props: RenderCfgProps) -> AnyView {
    /// #         view! { <div>{"Configuration UI"}</div> }.into_any()
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
    pub fn register(&mut self, plugin: Arc<dyn SportPortWebUi>) -> Result<()> {
        let plugin_id = plugin.get_id_version().get_id();
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
    /// # use app_core::{
    /// #   SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore,
    /// #   utils::{
    /// #       id_version::IdVersion,
    /// #       traits::ObjectIdVersion,
    /// #       validation::ValidationErrors,
    /// #   }
    /// # };
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::{SportPortWebUi, RenderCfgProps};
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl ObjectIdVersion for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, Some(0))
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig, _err: ValidationErrors) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportPortWebUi for MockSport {
    /// #     fn render_plugin_selection(&self) -> AnyView {
    /// #         view! { <div>{ "Mock Plugin Selection" }</div> }.into_any()
    /// #     }
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// #     fn render_dropdown(&self, config: &SportConfig) -> AnyView {
    /// #         view! { <div>{format!("Dropdown: {}", config.get_name())}</div> }.into_any()
    /// #     }
    /// #     fn render_configuration(&self, _props: RenderCfgProps) -> AnyView {
    /// #         view! { <div>{"Configuration UI"}</div> }.into_any()
    /// #     }
    /// # }
    /// #
    /// let mut manager = SportPluginManagerMap::new();
    /// let sport_id = Uuid::new_v4();
    /// let plugin = Arc::new(MockSport { id: sport_id, name: "MockSport" });
    /// manager.register(plugin).unwrap();
    ///
    /// // Get an existing plugin
    /// let found_plugin = manager.get_web_ui(&sport_id);
    /// assert!(found_plugin.is_some());
    /// assert_eq!(found_plugin.unwrap().get_id_version().get_id(), sport_id);
    ///
    /// // Try to get a non-existent plugin
    /// let not_found_plugin = manager.get_web_ui(&Uuid::new_v4());
    /// assert!(not_found_plugin.is_none());
    /// ```
    pub fn get_web_ui(&self, sport_id: &Uuid) -> Option<Arc<dyn SportPortWebUi>> {
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
    /// # use app_core::{
    /// #   SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore,
    /// #   utils::{
    /// #       id_version::IdVersion,
    /// #       traits::ObjectIdVersion,
    /// #       validation::ValidationErrors,
    /// #   }
    /// # };
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::{SportPortWebUi, RenderCfgProps};
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl ObjectIdVersion for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, Some(0))
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig, _err: ValidationErrors) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportPortWebUi for MockSport {
    /// #     fn render_plugin_selection(&self) -> AnyView {
    /// #         view! { <div>{ "Mock Plugin Selection" }</div> }.into_any()
    /// #     }
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// #     fn render_dropdown(&self, config: &SportConfig) -> AnyView {
    /// #         view! { <div>{format!("Dropdown: {}", config.get_name())}</div> }.into_any()
    /// #     }
    /// #     fn render_configuration(&self, _props: RenderCfgProps) -> AnyView {
    /// #         view! { <div>{"Configuration UI"}</div> }.into_any()
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
    /// assert_eq!(found_plugin.unwrap().get_id_version().get_id(), sport_id);
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
    /// # use app_core::{
    /// #   SportPort, SportPluginManagerPort, Match, SportConfig, SportResult, EntrantGroupScore,
    /// #   utils::{
    /// #       id_version::IdVersion,
    /// #       traits::ObjectIdVersion,
    /// #       validation::ValidationErrors,
    /// #   }
    /// # };
    /// # use std::{sync::Arc, time::Duration};
    /// # use uuid::Uuid;
    /// # use leptos::prelude::*;
    /// # use shared::{SportPortWebUi, RenderCfgProps};
    /// #
    /// # struct MockSport { id: Uuid, name: &'static str };
    /// # impl ObjectIdVersion for MockSport {
    /// #     fn get_id_version(&self) -> IdVersion {
    /// #         IdVersion::new(self.id, Some(0))
    /// #     }
    /// # }
    /// # impl SportPort for MockSport {
    /// #     fn name(&self) -> &'static str { self.name }
    /// #     fn get_default_config(&self) -> serde_json::Value { serde_json::json!({}) }
    /// #     fn validate_config_values(&self, _config: &SportConfig, _err: ValidationErrors) -> SportResult<()> { Ok(()) }
    /// #     fn estimate_match_duration(&self, _config: &SportConfig) -> SportResult<Duration> { Ok(Duration::from_secs(0)) }
    /// #     fn validate_final_score(&self, _config: &SportConfig, _score: &Match) -> SportResult<()> { Ok(()) }
    /// #     fn get_entrant_group_score(&self, _config: &SportConfig, group_id: Uuid, entrant_id: Uuid, _all_matches: &[Match]) -> SportResult<EntrantGroupScore> {
    /// #         Ok(EntrantGroupScore { entrant_id, group_id, victory_points: 0.0, relative_score: 0, total_score: 0 })
    /// #     }
    /// # }
    /// # impl SportPortWebUi for MockSport {
    /// #     fn render_plugin_selection(&self) -> AnyView {
    /// #         view! { <div>{ "Mock Plugin Selection" }</div> }.into_any()
    /// #     }
    /// #     fn render_preview(&self, _config: &SportConfig) -> AnyView {
    /// #         view! { <div>{ "Mock Preview" }</div> }.into_any()
    /// #     }
    /// #     fn render_dropdown(&self, config: &SportConfig) -> AnyView {
    /// #         view! { <div>{format!("Dropdown: {}", config.get_name())}</div> }.into_any()
    /// #     }
    /// #     fn render_configuration(&self, _props: RenderCfgProps) -> AnyView {
    /// #         view! { <div>{"Configuration UI"}</div> }.into_any()
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

// testing of SportPluginManagerMap is done in integration_testing crate -> sport fake
