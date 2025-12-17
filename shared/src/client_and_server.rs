//! Shared types and traits used by both client and server

use app_core::{SportConfig, SportPort};
use leptos::prelude::*;
use serde_json::Value;

/// Trait for rendering sport configuration in web ui
pub trait SportConfigWebUi: Send + Sync + SportPort {
    fn render_preview(&self, config: &SportConfig) -> AnyView;
    fn render_dropdown(&self, config: &SportConfig) -> AnyView;
    fn render_configuration(&self, props: RenderCfgProps) -> AnyView;
}

#[derive(Clone, Copy)]
pub struct RenderCfgProps {
    pub config: RwSignal<Option<Value>>,
    pub is_valid_json: RwSignal<bool>,
    pub is_new: Signal<bool>,
    pub is_loading: Signal<bool>,
}
