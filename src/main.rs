use dotenvy::dotenv;
use hyper::body::Bytes;
use jsonrpsee::{RpcModule, core::middleware::RpcServiceBuilder, server::Server};
use sea_orm::Database;
use std::{env, error::Error, iter::once, net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{
    LatencyUnit,
    compression::CompressionLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::EnvFilter;

mod context;
mod entities;
mod methods;
mod middleware;
mod models;

use crate::{
    context::GatewayContext,
    methods::pamplona::{PamplonaImpl, PamplonaServer},
    middleware::{
        http::{GATEWAY_SESSION_HEADER, HttpMiddlewareLayer},
        rpc::RpcMiddlewareLayer,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv().ok();

    // Set up logging based on the environment filter
    tracing_subscriber::FmtSubscriber::builder()
        // .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
        .with_env_filter(EnvFilter::new("debug"))
        .try_init()
        .expect("setting default subscriber failed");

    let db =
        &Database::connect(env::var("DATABASE_URL").expect("DATABASE_URL must be set")).await?;

    // Synchronize database schema with entity definitions
    db.get_schema_registry("gateway::entities::*")
        .sync(db)
        .await?;

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

    // In my understanding, the RpcModule context can only be used by
    // the methods registered directly via `register_method`, or `register_async_method`
    // So we will have to pass the context to the impls directly
    let mut methods: RpcModule<()> = RpcModule::new(());

    let context = Arc::new(GatewayContext::new());

    let pamplona_impl = PamplonaImpl::new(context.clone());
    methods.merge(pamplona_impl.into_rpc())?;

    // let pamplona_auth_impl = PamplonaAuthenticatedImpl::new(context.clone());
    // methods.merge(pamplona_auth_ipml.into_rpc())?;

    let handle = server.start(methods);
    handle.stopped().await;

    Ok(())
}
