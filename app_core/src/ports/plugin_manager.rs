//! Sport Plugin Manager Port

use crate::ports::sport::SportPort;
use std::{any::Any, sync::Arc};
use uuid::Uuid;

pub trait SportPluginManagerPort: Send + Sync + Any {
    fn get(&self, sport_id: &Uuid) -> Option<Arc<dyn SportPort>>;
    fn list(&self) -> Vec<Arc<dyn SportPort>>;
}