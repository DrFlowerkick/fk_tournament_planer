// hook for leptos to use the sse api

use app_core::{CrPushNotice, CrTopic};
use crate::SseUrl;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use codee::string::JsonSerdeCodec;
use leptos::{logging::log, prelude::*};

pub fn use_changed_sse(topic: CrTopic, refetch: impl Fn() + 'static, version: impl Fn() -> i64 + 'static) {
    
    let UseEventSourceReturn {
        data,
        ..
    } = use_event_source_with_options::<CrPushNotice, JsonSerdeCodec>(
        topic.sse_url().as_str(),
        UseEventSourceOptions::default().named_events(["changed".to_string()]),
    );

    Effect::new(move || {
        if let Some(data) = data.get() {
            match data {
                CrPushNotice::AddressUpdated { id, meta } => {
                    // id should always be equal, since we subscribed for topic id
                    if *topic.id() == id && meta.version > version() {
                        log!("refetching id: {}", id);
                        refetch();
                    }
                }
            }
        }
    });
}
