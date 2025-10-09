use app::*;
use app_core::*;
use axum::Router;
use axum_extra::routing::RouterExt;
use cr_single_instance::*;
use db_postgres::*;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use shared::*;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // initialize core state
    let core = CoreBuilder::new()
        .set_db(Arc::new(PgDb::new().await.unwrap()))
        .set_cr(Arc::new(CrSingleInstance::new()))
        .build();
    let app_state = AppState {
        core: Arc::new(core),
        leptos_options: leptos_options.clone(),
    };
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .typed_get(api_subscribe)
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
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
