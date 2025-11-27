//! Sport Plugin Manager Port

use crate::ports::sport::SportPort;
use std::{any::Any, sync::Arc};
use uuid::Uuid;

pub trait SportPluginManagerPort: Send + Sync + Any {
    fn get(&self, sport_id: &Uuid) -> Option<Arc<dyn SportPort>>;
    fn list(&self) -> Vec<Arc<dyn SportPort>>;
}

// ToDo: in a later stage, Sport Plug-Ins should be dynamically loadable
// For now, we can implement a simple in-memory manager
// that holds a list of statically registered sport plugins.
