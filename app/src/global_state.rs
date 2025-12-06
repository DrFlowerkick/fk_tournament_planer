//! global state management in reactive system of app

use generic_sport_plugin::GenericSportPlugin;
use reactive_stores::Store;
use sport_plugin_manager::SportPluginManagerMap;
use std::sync::Arc;

#[derive(Clone, Store)]
pub struct GlobalState {
    /// path before new/edit address
    pub return_after_address_edit: String,
    /// path before new/edit sport config
    pub return_after_sport_config_edit: String,
    /// sport plugin manager
    pub sport_plugin_manager: SportPluginManagerMap,
}

// list of registered sport plugins must match the one in server initialization
impl GlobalState {
    pub fn new() -> Self {
        let mut sport_plugin_manager = SportPluginManagerMap::new();
        sport_plugin_manager
            .register(Arc::new(GenericSportPlugin::new()))
            .unwrap();
        GlobalState {
            return_after_address_edit: "/".to_string(),
            return_after_sport_config_edit: "/".to_string(),
            sport_plugin_manager,
        }
    }
}
