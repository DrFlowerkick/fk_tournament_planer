// client registry based upon leptos-axum-socket

use anyhow::Result as AnyResult;
use app_core::{ClientRegistryPort, CrMsg, CrTopic};
use async_trait::async_trait;
#[cfg(feature = "ssr")]
use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum_socket::{ServerSocket, handlers::upgrade_websocket};
use leptos_axum_socket::{SocketMsg, expect_socket_context};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use shared::AppState;
use tracing::{info, instrument};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CrSocketMsg {
    pub msg: CrMsg,
}

impl SocketMsg for CrSocketMsg {
    type Key = CrTopic;
    #[cfg(feature = "ssr")]
    type AppState = AppState;
}

#[derive(Clone)]
pub struct ClientRegistrySocket {}

#[cfg(feature = "ssr")]
#[async_trait]
impl ClientRegistryPort for ClientRegistrySocket {
    #[instrument(
        name = "cr_socket.publish",
        skip_all,
        fields(topic = %topic)
    )]
    async fn publish(&self, topic: CrTopic, msg: CrMsg) -> AnyResult<()> {
        let msg = CrSocketMsg { msg };
        //leptos_axum_socket::send(&topic, &msg).await;
        info!(
            "THIS IS NEVER CALLED: Publishing to topic {}: {:?}",
            topic, msg
        );
        Ok(())
    }
}

// Implement the `connect_to_websocket` handler:
#[cfg(feature = "ssr")]
pub async fn connect_to_websocket(
    ws: WebSocketUpgrade,
    State(socket): State<ServerSocket>,
) -> Response {
    // You could do authentication here

    // Provide extra context like the user's ID for example that is passed to the permission filters
    let ctx = ();

    upgrade_websocket(ws, socket, ctx)
}

// client registry subscription hook for leptos components
pub fn use_client_registry_topic(
    topic: ReadSignal<Option<CrTopic>>,
    handler: impl Fn(&CrSocketMsg) + Clone + Send + Sync + 'static,
) {
    let socket = expect_socket_context();

    Effect::new(move || {
        if let Some(topic) = topic.get() {
            socket.subscribe(topic, handler.clone());
        }
    });
}
