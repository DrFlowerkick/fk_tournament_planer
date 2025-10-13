// shared data types, used by server and client

use app_core::CoreServerState;
use axum_macros::FromRef;
use leptos::prelude::*;

// todo: if this is never used in app, it may be moved to server
#[derive(FromRef, Clone)]
pub struct AppState {
    pub core: CoreServerState,
    pub leptos_options: LeptosOptions,
}
