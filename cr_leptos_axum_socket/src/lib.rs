// client registry based upon leptos-axum-socket

use anyhow::Result as AnyResult;
use app_core::{ClientRegistryPort, CrMsg, CrTopic};
use async_trait::async_trait;
#[cfg(feature = "ssr")]
use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use leptos::logging::log;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum_socket::{ServerSocket, handlers::upgrade_websocket};
use leptos_axum_socket::{SocketMsg, expect_socket_context};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use shared::AppState;
use std::sync::Arc;

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
    version: ReadSignal<u32>,
    refetch: Arc<dyn Fn() + Send + Sync + 'static>,
) {
    #[cfg(feature = "hydrate")]
    {
        let socket = expect_socket_context();

        let subscribe = {
            move |topic: CrTopic, refetch: Arc<dyn Fn() + Send + Sync + 'static>| {
                let version = version.get_untracked();
                let socket_handler = move |msg: &CrSocketMsg| match msg.msg {
                    CrMsg::AddressUpdated {
                        version: meta_version,
                        ..
                    } => {
                        if meta_version > version {
                            log!(
                                "AddressUpdated received: refetching address expecting version: {}",
                                meta_version
                            );
                            refetch();
                        }
                    }
                };
                log!("Subscribing to topic: {:?}", topic);
                socket.subscribe(topic, socket_handler);
            }
        };

        Effect::watch(
            move || topic.get(),
            move |tp, prev_tp, _| {
                if let Some(topic) = tp {
                    if let Some(Some(prev_topic)) = prev_tp
                        && prev_topic != topic
                    {
                        log!("Unsubscribing from previous topic: {:?}", prev_topic);
                        socket.unsubscribe(prev_topic);
                    }
                    subscribe(*topic, refetch.clone());
                }
            },
            true,
        );
    }
}

/// hook for subscribing to client registry
pub fn subscribe_topic_version(
    topic: CrTopic,
    version: ReadSignal<u32>,
    refetch: impl Fn() + Clone + Send + Sync + 'static,
) {
    let socket = expect_socket_context();
    let version = version.get_untracked();

    let socket_handler = move |msg: &CrSocketMsg| match msg.msg {
        CrMsg::AddressUpdated {
            version: meta_version,
            ..
        } => {
            if meta_version > version {
                log!(
                    "AddressUpdated received: refetching address expecting version: {}",
                    meta_version
                );
                refetch();
            }
        }
    };

    socket.subscribe(topic, socket_handler);
}
