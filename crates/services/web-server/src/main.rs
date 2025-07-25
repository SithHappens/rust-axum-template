#![allow(unused)] // For early development.

mod config;
mod error;
mod log;
mod web;

pub use self::error::{Error, Result};

use crate::web::mw_auth::{mw_ctx_require, mw_ctx_resolver};
use crate::web::mw_res_map::mw_response_map;
use crate::web::{routes_login, routes_rpc, routes_static};
use axum::response::Html;
use axum::routing::get;
use axum::{Router, middleware};
use lib_core::_dev_utils;
use lib_core::model::ModelManager;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time() // for early development
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // FOR DEV ONLY
    _dev_utils::init_dev().await;

    // Initialize ModelManager.
    let mm = ModelManager::new().await?;

    // -- Define Routes
    let routes_rpc =
        routes_rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

    // Called in the order from bottom to top, this is why the Cookie Manager needs to be after
    // bottom, so all the top layers can use them.
    let routes_all = Router::new()
        .merge(routes_login::routes(mm.clone()))
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolver))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}
