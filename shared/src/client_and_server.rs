//! Shared types and traits used by both client and server

use app_core::{SportConfig, SportPort};
use leptos::prelude::*;

/// Trait for rendering sport port specifics in web ui, e.g., configuration forms and previews.
pub trait SportPortWebUi: Send + Sync + SportPort {
    fn render_plugin_selection(&self) -> AnyView;
    fn render_preview(&self, config: &SportConfig) -> AnyView;
    fn render_dropdown(&self, config: &SportConfig) -> AnyView;
    fn render_configuration(&self) -> AnyView;
}
