//! Global state management for the application

use reactive_stores::Store;
use sport_plugin_manager::SportPluginManagerMap;

// ToDo: We keep GlobalState for now, because we probably will use i for user management.
// But if not, we will change it to context provided signal, equal to error and toast context.
#[derive(Clone, Store)]
pub struct GlobalState {
    /// sport plugin manager
    pub sport_plugin_manager: SportPluginManagerMap,
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            sport_plugin_manager: SportPluginManagerMap::new(),
        }
    }
}
