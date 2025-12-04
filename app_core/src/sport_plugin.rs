//! Sport Plugin core functionality

use crate::{Core, SportPort};
use std::sync::Arc;
use uuid::Uuid;

/// State marker for SportPlugin related functionality
pub struct SportPluginState {}

// switch state to sport plugin state
impl<S> Core<S> {
    pub fn as_sport_plugin_state(&self) -> Core<SportPluginState> {
        self.switch_state(SportPluginState {})
    }
}

impl Core<SportPluginState> {
    pub fn list_sports(&self) -> Vec<Arc<dyn SportPort>> {
        self.sport_plugins.list()
    }
    pub fn get_sport(&self, sport_id: &Uuid) -> Option<Arc<dyn SportPort>> {
        self.sport_plugins.get(sport_id)
    }
}
