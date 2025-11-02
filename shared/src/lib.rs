// shared data types, used by server and client

use app_core::CoreState;
use axum_macros::FromRef;
use cr_single_instance::CrSingleInstance;
use leptos::prelude::*;
use leptos_axum_socket::ServerSocket;
use std::sync::Arc;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub core: CoreState,
    pub leptos_options: LeptosOptions,
    pub socket: ServerSocket,
    pub cr_single_instance: Arc<CrSingleInstance>,
}
