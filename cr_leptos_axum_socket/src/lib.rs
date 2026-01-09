// client registry based upon leptos-axum-socket

#[cfg(feature = "ssr")]
use app_core::{ClientRegistryPort, CrResult};
use app_core::{CrMsg, CrTopic};
#[cfg(feature = "ssr")]
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
#[cfg(feature = "ssr")]
use tracing::instrument;

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
    #[instrument(name = "cr.publish", skip(self, msg))]
    async fn publish(&self, topic: CrTopic, msg: CrMsg) -> CrResult<()> {
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
pub fn use_client_registry_socket(
    topic: ReadSignal<Option<CrTopic>>,
    version: ReadSignal<u32>,
    refetch: Arc<dyn Fn() + Send + Sync + 'static>,
) {
    // ToDo: clean up logs
    let socket = expect_socket_context();

    let prev_topic = StoredValue::new(None::<CrTopic>);

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
                CrMsg::SportConfigUpdated {
                    version: meta_version,
                    ..
                } => {
                    if meta_version > version {
                        log!(
                            "SportConfigUpdated received: refetching config expecting version: {}",
                            meta_version
                        );
                        refetch();
                    }
                }
                CrMsg::TournamentBaseUpdated {
                    version: meta_version,
                    ..
                } => {
                    if meta_version > version {
                        log!(
                            "TournamentBaseUpdated received: refetching tournament expecting version: {}",
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
        move |tp, _, _| {
            log!("Topic changed: {:?}", tp);
            log!("Previous topic: {:?}", prev_topic.get_value());
            if let Some(topic) = tp {
                if let Some(prev_tp) = prev_topic.get_value()
                    && prev_tp != *topic
                {
                    log!("Unsubscribing from previous topic: {:?}", prev_tp);
                    socket.unsubscribe(prev_tp);
                    subscribe(*topic, refetch.clone());
                    prev_topic.set_value(Some(*topic));
                } else if prev_topic.get_value().is_none() {
                    subscribe(*topic, refetch.clone());
                    prev_topic.set_value(Some(*topic));
                }
            }
        },
        true,
    );

    on_cleanup(move || {
        if let Some(topic) = topic.get_untracked() {
            log!("Cleaning up subscription for topic: {:?}", topic);
            socket.unsubscribe(topic);
        }
    });
}
