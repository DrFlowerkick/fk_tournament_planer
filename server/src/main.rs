mod sse_service;

use anyhow::Result;
use app::*;
use app_core::*;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use cr_leptos_axum_socket::{ClientRegistrySocket, connect_to_websocket};
use db_postgres::*;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use leptos_axum_socket::{ServerSocket, SocketRoute};
use serde::Serialize;
use shared::*;
use std::sync::Arc;
use tracing::{info, instrument};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};

use anyhow::Context;
use std::env;

fn init_tracing_bunyan() -> Result<()> {
    // Read level configuration from env (.env via dotenvy or docker sets env)
    let rust_log =
        env::var("RUST_LOG").context("POSTGRES_URL must be set. Hint: did you run dotenv()?")?;
    let database_name = env::var("DATABASE_NAME")
        .context("POSTGRES_URL must be set. Hint: did you run dotenv()?")?;
    dbg!(rust_log, database_name);
    let env_filter =
        //EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,axum=info"));
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("error,axum=error"));

    // Name identifies the service in log streams (use your app/service name)
    let formatting_layer = BunyanFormattingLayer::new(
        "tournament-app".into(),
        std::io::stdout, // single sink: JSON to stdout; no other outputs supported
    );

    // Build a Bunyan-only subscriber:
    // - JsonStorageLayer: propagates span fields to child events
    // - BunyanFormattingLayer: strict Bunyan JSON output
    // - ErrorLayer: enrich errors with span context
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(ErrorLayer::default());

    // Set as the single global subscriber (no fallback to fmt/console)
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

// --- /health (service liveness) ---
#[instrument(name = "health")]
async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

// --- /health/db (database readiness) ---
#[derive(Serialize)]
struct DbStatus {
    db: &'static str,
}

#[instrument(name = "health_db", skip(app_state))]
async fn health_db(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state.core.database.ping_db().await {
        Ok(_) => (StatusCode::OK, axum::Json(DbStatus { db: "ok" })),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(DbStatus { db: "down" }),
        ),
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Load .env first if present; ignore if missing (Docker sets envs)
    dotenvy::dotenv()?;
    // map all log! calls in dependencies to tracing
    LogTracer::init()?;
    // Initialize Bunyan-only tracing before constructing anything else.
    init_tracing_bunyan()?;

    // load leptos options
    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // initialize core state
    let db = PgDb::new(url_db()?).await?;
    db.run_migration().await?;
    let cr = Arc::new(ClientRegistrySocket {});
    let core = CoreBuilder::new()
        .set_db(Arc::new(db))
        .set_cr(cr.clone())
        .build();
    let app_state = AppState {
        core: Arc::new(core),
        leptos_options: leptos_options.clone(),
        socket: ServerSocket::new(),
    };
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/health", get(health))
        .route("/health/db", get(health_db))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let core = app_state.core.clone();
                move || provide_context(core.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .socket_route(connect_to_websocket)
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    info!(%addr, "listening on http server");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
