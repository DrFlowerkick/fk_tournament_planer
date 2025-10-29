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
    async fn publish(&self, topic: CrTopic, msg: CrMsg) -> AnyResult<()> {
        let msg = CrSocketMsg { msg };
        leptos_axum_socket::send(&topic, &msg).await;
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

    //let prev_topic = StoredValue::new(None::<CrTopic>);

    Effect::new(move || {
        let socket = expect_socket_context();
        //    if let Some(topic) = prev_topic.get_value() {
        //        socket.unsubscribe(topic);
        //        prev_topic.set_value(None);
        //    }
        if let Some(topic) = topic.get() {
            //        prev_topic.set_value(Some(topic));
            socket.subscribe(topic, handler.clone());
        }
    });

    //on_cleanup(move || socket.unsubscribe(topic.get_untracked()));
}
