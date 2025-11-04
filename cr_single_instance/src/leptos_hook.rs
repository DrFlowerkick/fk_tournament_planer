use app_core::CrMsg;
use codee::string::JsonSerdeCodec;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::web_sys::Event;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use std::sync::Arc;

pub fn use_client_registry_sse(
    url: ReadSignal<String>,
    version: ReadSignal<u32>,
    refetch: Arc<dyn Fn() + Send + Sync + 'static>,
) {
    Effect::new(move || {
        let url = url.get();
        log!("ClientRegistry SSE URL: {}", url);
    });

    let changed_handler = move |event: &Event| {
        log!(
            "ClientRegistry SSE custom Changed Event received: {}",
            event.type_()
        );
        false
    };

    let UseEventSourceReturn { data, .. } = use_event_source_with_options::<CrMsg, JsonSerdeCodec>(
        url,
        UseEventSourceOptions::default()
            .immediate(true)
            .named_events(["changed".to_string()])
            .pre_event_handler(changed_handler),
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
