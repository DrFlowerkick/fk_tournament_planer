// web service helper functions and types to use the client registry in web

use crate::CrKind;
use app_core::{CrMsg, CrTopic};
use axum::{
    extract::State,
    response::{Sse, sse::Event},
};
use axum_extra::routing::TypedPath;
use futures_core::Stream;
use futures_util::StreamExt;
use serde::Deserialize;
use shared::AppState;
use std::convert::Infallible;
use tokio_stream::once;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

// typed_path must match to crate::types::CR_TOPIC_URL_TEMPLATE
#[derive(TypedPath, Deserialize, Clone, Copy)]
#[typed_path("/api/cr/subscribe/{kind}/{id}")]
pub struct CrTopicPath {
    kind: CrKind,
    id: Uuid,
}

impl From<CrTopicPath> for CrTopic {
    fn from(value: CrTopicPath) -> Self {
        match value.kind {
            CrKind::Address => CrTopic::Address(value.id),
        }
    }
}

/// SSE entrypoint (typed route). We add a per-connection span for better correlation.
/// Fields like `topic` or `client_ip` are valuable to debug fan-out.
#[instrument(name = "sse_connection", skip(state), fields(topic = %topic))]
pub async fn api_subscribe(
    topic: CrTopicPath,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("SSE connected");
    let topic = CrTopic::from(topic);

    let out = match state.cr_single_instance.subscribe(topic).await {
        Ok(st) => st
            .map(|changed| {
                let CrMsg::AddressUpdated { id, .. } = &changed;
                match serde_json::to_string(&changed) {
                    Ok(s) => Ok(Event::default().event("changed").data(s).id(id.to_string())),
                    Err(e) => {
                        // recoverable per-event failure: warn (don’t spam)
                        warn!(error = %e, "serialize_changed_failed");
                        Ok(Event::default()
                            .event("error")
                            .data(format!("serde error: {e}")))
                    }
                }
            })
            .boxed(),
        Err(e) => {
            // subscription failed: the primary goal of the endpoint failed -> error
            error!(error = %e, "subscribe_failed");
            once(Ok(Event::default()
                .event("error")
                .data(format!("subscribe failed: {e}"))))
            .boxed()
        }
    };

    Sse::new(out).keep_alive(axum::response::sse::KeepAlive::default())
}
