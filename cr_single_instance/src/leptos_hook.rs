#[cfg(not(feature = "ssr"))]
use crate::types::SseUrl;
#[cfg(not(feature = "ssr"))]
use app_core::CrMsg;
use app_core::CrTopic;
#[cfg(not(feature = "ssr"))]
use codee::string::JsonSerdeCodec;
#[cfg(not(feature = "ssr"))]
use leptos::logging::log;
use leptos::prelude::*;
#[cfg(not(feature = "ssr"))]
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use std::sync::Arc;

pub fn use_client_registry_sse(
    topic: ReadSignal<Option<CrTopic>>,
    version: ReadSignal<u32>,
    refetch: Arc<dyn Fn() + Send + Sync + 'static>,
) {
    #[cfg(feature = "ssr")]
    {
        let _ = topic;
        let _ = version;
        let _ = refetch;
    }
    #[cfg(not(feature = "ssr"))]
    {
        let url = Signal::derive(move || topic.get().map(|t| t.sse_url()).unwrap_or_default());

        Effect::new(move || {
            let url = url.get();
            log!("ClientRegistry SSE URL: {}", url);
        });

        let UseEventSourceReturn { data, .. } =
            use_event_source_with_options::<CrMsg, JsonSerdeCodec>(
                url,
                UseEventSourceOptions::default()
                    .immediate(false)
                    .named_events(["changed".to_string()]),
            );

        Effect::new(move || {
            if let Some(event) = data.get() {
                match event {
                    CrMsg::AddressUpdated {
                        version: meta_version,
                        ..
                    } => {
                        log!("ClientRegistry SSE Event version: {}", meta_version);
                        if meta_version > version.get_untracked() {
                            refetch();
                        }
                    }
                }
            }
        });
    }
}
