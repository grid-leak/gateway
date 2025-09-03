use dotenvy::dotenv;
use gateway::establish_db_connection;
use hyper::body::Bytes;
use jsonrpsee::{RpcModule, core::middleware::RpcServiceBuilder, server::Server};
use tracing::info;
use std::{error::Error, iter::once, net::SocketAddr, time::Duration};
use tower_http::{
    LatencyUnit,
    compression::CompressionLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::EnvFilter;

mod methods;
mod middleware;
mod models;
mod schema;

use crate::{
    methods::{
        pamplona::{PamplonaImpl, PamplonaServer},
        pamplona_authenticated::{PamplonaAuthenticatedImpl, PamplonaAuthenticatedServer},
    },
    middleware::{
        http::{GATEWAY_SESSION_HEADER, HttpMiddlewareLayer},
        rpc::RpcMiddlewareLayer,
    },
};

use self::models::*;
use diesel::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use self::schema::users::dsl::*;

    // Load environment variables
    dotenv().ok();

    let connection = &mut establish_db_connection();
    let results = users
        .select(User::as_select())
        .load(connection)
        .expect("Error loading users");

    info!("Loaded {} users", results.len());

    // Set up logging based on the environment filter
    tracing_subscriber::FmtSubscriber::builder()
        // .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
        .with_env_filter(EnvFilter::new("debug"))
        .try_init()
        .expect("setting default subscriber failed");

    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    // High level tracing/logging for all requests
    let trace_layer =TraceLayer::new_for_http()
        .on_request(
            |request: &hyper::Request<_>, _span: &tracing::Span| tracing::debug!(request = ?request, "on_request")
        )
        .on_body_chunk(
            |chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                tracing::info!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
        })
        .make_span_with(DefaultMakeSpan::new())
        .on_response(DefaultOnResponse::new().latency_unit(LatencyUnit::Micros));

    let service_builder = tower::ServiceBuilder::new()
        .layer(HttpMiddlewareLayer::new())
        .layer(SetSensitiveRequestHeadersLayer::new(once(
            GATEWAY_SESSION_HEADER,
        )))
        .layer(CompressionLayer::new())
        .layer(trace_layer);

    let rpc_middleware = RpcServiceBuilder::new().layer(RpcMiddlewareLayer);

    let server = Server::builder()
        .set_http_middleware(service_builder)
        .set_rpc_middleware(rpc_middleware)
        .build(addr)
        .await?;

    // TODO: add a database connection pool to the Context
    // Context is shared across all connections and requests,
    // making it suitable for a global state like this
    let mut methods = RpcModule::new(());
    methods.merge(PamplonaAuthenticatedImpl.into_rpc())?;
    methods.merge(PamplonaImpl.into_rpc())?;

    let handle = server.start(methods);
    handle.stopped().await;

    Ok(())
}
