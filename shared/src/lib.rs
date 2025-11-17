// shared data types, used by server and client

use app_core::CoreState;
use axum_macros::FromRef;
use leptos::prelude::*;
use leptos_axum_socket::ServerSocket;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub core: CoreState,
    pub leptos_options: LeptosOptions,
    pub socket: ServerSocket,
}
