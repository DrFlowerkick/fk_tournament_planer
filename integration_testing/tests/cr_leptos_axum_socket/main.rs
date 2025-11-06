// Integration test for cr_leptos_axum_socket
// This test checks the publish and receive logic via WebSocket
// Note: This is a draft and may require additional setup/mocks for full functionality

use tokio::sync::oneshot;
use app_core::{CrMsg, CrTopic, CoreBuilder};
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::Router;
use axum::routing::get;
use std::net::SocketAddr;
use tokio::task;
use uuid::Uuid;
use shared::AppState;
use leptos_axum_socket::{ServerSocket, SocketRoute};
use cr_leptos_axum_socket::{connect_to_websocket, ClientRegistrySocket};
use std::sync::Arc;
use leptos::prelude::*;
use integration_testing::port_fakes::FakeDatabasePort;


#[tokio::test]
async fn test_publish_and_receive_message() {
    // Setup: Start a test server with WebSocket endpoint
    let db = FakeDatabasePort::new();
    let cr = Arc::new(ClientRegistrySocket {});
    let core = CoreBuilder::new()
        .set_db(Arc::new(db))
        .set_cr(cr.clone())
        .build();

    /*let leptos_options = LeptosOptions::builder()
        .site_addr("127.0.0.1:0")
        .output_name("test axum socket")
        .build();
    let app_state = AppState {
        core: Arc::new(core),
        leptos_options,
        socket: ServerSocket::new(),
    };
    let app = Router::new()
        .socket_route(connect_to_websocket)
        .with_state(app_state);*/

    // Simulate a topic and message
    let id = Uuid::new_v4();
    let topic = CrTopic::Address(id);
    let msg = CrMsg::AddressUpdated { id, version: 2 };

    // Simulate publish
    // let result = ClientRegistrySocket.publish(topic.clone(), msg.clone()).await;
    // assert!(result.is_ok());

    // Simulate client receiving the message
    // Here you would connect a WebSocket client and check for the message
    // This part requires a running server and client implementation

    // Example assertion (to be replaced with actual logic):
    // assert_eq!(received_msg, msg);
}
