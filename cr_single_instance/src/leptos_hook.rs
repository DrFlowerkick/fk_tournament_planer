use crate::SseUrl;
use app_core::{CrMsg, CrTopic};
use codee::string::JsonSerdeCodec;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::web_sys::Event;
use leptos_use::{
    UseEventSourceOnEventReturn, UseEventSourceOptions, UseEventSourceReturn,
    use_event_source_with_options,
};
use std::sync::Arc;

pub fn use_client_registry_sse(
    topic: ReadSignal<Option<CrTopic>>,
    version: ReadSignal<u32>,
    refetch: Arc<dyn Fn() + Send + Sync + 'static>,
) {
    let url = Signal::derive(move || topic.get().map(|t| t.sse_url()).unwrap_or_default());

    // ToDo: to clean up: remove url effect and log!
    Effect::new(move || {
        let url = url.get();
        log!("ClientRegistry SSE URL: {}", url);
    });

    let changed_handler = move |event: &Event| {
        log!(
            "ClientRegistry SSE custom Changed Event received: {}",
            event.type_()
        );
        UseEventSourceOnEventReturn::ProcessMessage
    };

    let UseEventSourceReturn { message, close, .. } =
        use_event_source_with_options::<CrMsg, JsonSerdeCodec>(
            url,
            UseEventSourceOptions::default()
                .immediate(true)
                .named_events(["changed".to_string()])
                .on_event(changed_handler),
        );

    Effect::new(move || {
        if let Some(event) = message.get() {
            match event.data {
                CrMsg::AddressUpdated {
                    version: meta_version,
                    ..
                } => {
                    log!("ClientRegistry SSE Event version: {}", meta_version);
                    if meta_version > version.get_untracked() {
                        refetch();
                    }
                }
                CrMsg::SportConfigUpdated {
                    version: meta_version,
                    ..
                } => {
                    log!("ClientRegistry SSE Event version: {}", meta_version);
                    if meta_version > version.get_untracked() {
                        refetch();
                    }
                }
                CrMsg::TournamentBaseUpdated {
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

    on_cleanup(move || {
        close();
    });
}
