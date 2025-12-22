//! global state management in reactive system of app

use reactive_stores::Store;
use sport_plugin_manager::SportPluginManagerMap;

#[derive(Clone, Store)]
pub struct GlobalState {
    /// path before new/edit address
    pub return_after_address_edit: String,
    /// path before new/edit sport config
    pub return_after_sport_config_edit: String,
    /// sport plugin manager
    pub sport_plugin_manager: SportPluginManagerMap,
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            return_after_address_edit: "/".to_string(),
            return_after_sport_config_edit: "/".to_string(),
            sport_plugin_manager: SportPluginManagerMap::new(),
        }
    }
}
