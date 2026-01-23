use crate::SseUrl;
use app_core::{CrMsg, CrTopic};
use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use std::sync::Arc;

pub fn use_client_registry_sse(
    topic: ReadSignal<Option<CrTopic>>,
    version: ReadSignal<u32>,
    refetch: Arc<dyn Fn() + Send + Sync + 'static>,
) {
    let url = Signal::derive(move || topic.get().map(|t| t.sse_url()).unwrap_or_default());
    let UseEventSourceReturn { message, close, .. } =
        use_event_source_with_options::<CrMsg, JsonSerdeCodec>(
            url,
            UseEventSourceOptions::default()
                .immediate(true)
                .named_events(["changed".to_string()]),
        );

    Effect::new(move || {
        if let Some(event) = message.get()
            && event.data.version() > version.get_untracked()
        {
            refetch();
        }
    });

    on_cleanup(move || {
        close();
    });
}
