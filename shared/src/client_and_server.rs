//! Shared types and traits used by both client and server

use app_core::{SportConfig, SportPort};
use leptos::prelude::*;

/// Trait for rendering sport configuration in web ui
pub trait SportConfigWebUi: Send + Sync + SportPort {
    fn render_preview(&self, config: &SportConfig) -> AnyView;
    fn render_dropdown(&self, config: &SportConfig) -> AnyView;
    fn render_configuration(&self) -> AnyView;
}
