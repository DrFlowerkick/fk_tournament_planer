//! Implementation of sport preview for the generic sport plugin

use super::GenericSportPlugin;
use app_core::SportConfig;
use leptos::prelude::*;
use shared::SportConfigWebUi;

impl SportConfigWebUi for GenericSportPlugin {
    fn render_preview(&self, config: &SportConfig) -> AnyView {
        let generic_config = match self.validate_config(config) {
            Ok(cfg) => cfg,
            Err(_) => return view! { <div>{"Invalid Configuration"}</div> }.into_any(),
        };
        view! {
            <div class="p-4 border rounded bg-white shadow-sm" data-testid="sport-config-preview">
                <p class="text-lg font-semibold mb-2">{config.name.clone()}</p>
                <p class="mb-1">
                    <strong>{"Expected Match Duration: "}</strong>
                    {format!(
                        "{} minutes",
                        generic_config.expected_match_duration_minutes.as_secs() / 60,
                    )}
                </p>
            </div>
        }
        .into_any()
    }
    fn render_dropdown(&self, config: &SportConfig) -> AnyView {
        view! {
            <div class="p-2" data-testid="sport-config-dropdown">
                <span class="font-medium">{config.name.clone()}</span>
            </div>
        }
        .into_any()
    }
    fn render_configuration(&self) -> AnyView {
        view! {
            <div class="p-4" data-testid="sport-config-configuration">
                <p>{"Generic Sport Plugin Configuration UI"}</p>
            </div>
        }
        .into_any()
    }
}
