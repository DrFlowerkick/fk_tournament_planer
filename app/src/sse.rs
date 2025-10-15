// generic sse listener

use app_core::{CrPushNotice, CrTopic};
use codee::string::JsonSerdeCodec;
use cr_single_instance::SseUrl;
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};

#[component]
pub fn sse_listener(
    // topic to subscribe to
    topic: CrTopic,
    // current version of object as derived signal
    version: impl Fn() -> u32 + 'static,
    // refetch of data source as derived signal
    refetch: impl Fn() + 'static,
) -> impl IntoView {
    let url = topic.sse_url();
    let UseEventSourceReturn { data, .. } =
        use_event_source_with_options::<CrPushNotice, JsonSerdeCodec>(
            url.as_str(),
            UseEventSourceOptions::default().named_events(["changed".to_string()]),
        );
    Effect::new(move || {
        if let Some(data) = data.get() {
            match data {
                CrPushNotice::AddressUpdated { id, meta } => {
                    // id should always be equal, since we subscribed for topic id
                    assert_eq!(*topic.id(), id);
                    if meta.version > version() {
                        refetch();
                    }
                }
            }
        }
    });
}
