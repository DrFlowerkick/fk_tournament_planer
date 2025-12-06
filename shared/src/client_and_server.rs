//! Shared types and traits used by both client and server

use app_core::{SportConfig, SportPort};
use leptos::prelude::*;

/// Trait for rendering a preview of a sport configuration
pub trait SportConfigPreview: Send + Sync + SportPort {
    fn render_preview(&self, config: &SportConfig) -> AnyView;
}
