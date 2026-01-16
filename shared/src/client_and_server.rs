//! Shared types and traits used by both client and server

use app_core::{SportConfig, SportPort};
use leptos::prelude::*;
use serde_json::Value;

/// Trait for rendering sport port specifics in web ui, e.g., configuration forms and previews.
pub trait SportPortWebUi: Send + Sync + SportPort {
    fn render_plugin_selection(&self) -> AnyView;
    fn render_preview(&self, config: &SportConfig) -> AnyView;
    fn render_dropdown(&self, config: &SportConfig) -> AnyView;
    fn render_configuration(&self, props: RenderCfgProps) -> AnyView;
}

#[derive(Clone, Copy)]
pub struct RenderCfgProps {
    pub config: RwSignal<Option<Value>>,
    pub is_valid_json: RwSignal<bool>,
    pub is_new: Signal<bool>,
}
