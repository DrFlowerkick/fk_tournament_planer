//! Implementation of sport preview for the generic sport plugin

use super::GenericSportPlugin;
use app_core::SportConfig;
use leptos::prelude::*;
use shared::SportConfigPreview;

impl SportConfigPreview for GenericSportPlugin {
    fn render_preview(&self, config: &SportConfig) -> AnyView {
        let generic_config = match self.validate_config(config) {
            Ok(cfg) => cfg,
            Err(_) => return view! { <div>{ "Invalid Configuration" }</div> }.into_any(),
        };
        view! {
            <div class="p-4 border rounded bg-white shadow-sm" data-testid="sport-config-preview">
                <p class="text-lg font-semibold mb-2">{ config.name.clone() }</p>
                <p class="mb-1"><strong>{ "Expected Match Duration: " }</strong>{ format!("{} minutes", generic_config.expected_match_duration_minutes.as_secs() / 60) }</p>
            </div>
        }.into_any()
    }
}
