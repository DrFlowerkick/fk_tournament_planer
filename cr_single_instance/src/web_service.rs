// web service helper functions and types to use the client registry in web

use app_core::CrTopic;
use axum::{
    extract::State,
    response::{Sse, sse::Event},
};
use axum_extra::routing::TypedPath;
use futures_core::Stream;
use futures_util::StreamExt;
use serde::Deserialize;
use shared::AppState;
use tokio_stream::once;
use uuid::Uuid;
use std::convert::Infallible;
use crate::CrKind;

// typed_path must match to crate::types::CR_TOPIC_URL_TEMPLATE
#[derive(TypedPath, Deserialize)]
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

pub async fn api_subscribe(
    topic: CrTopicPath,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let topic = CrTopic::from(topic);

    let out = match state.core.client_registry.subscribe(topic.clone()).await {
        Ok(st) => st
            .map(|changed| {
                match serde_json::to_string(&changed) {
                    Ok(s) => Ok(Event::default().event("changed").data(s)),
                    Err(e) => Ok(Event::default().event("error").data(format!("serde error: {e}"))),
                }
            })
            .boxed(),
        Err(e) => once(
            Ok(Event::default().event("error").data(format!("subscribe failed: {e}")))
        ).boxed(),
    };

    Sse::new(out).keep_alive(axum::response::sse::KeepAlive::default())
}
