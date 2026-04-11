use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Request, Response, StatusCode};
use jsonrpsee::server::HttpBody;
use sea_orm::EntityTrait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tower_http::BoxError;
use uuid::Uuid;

use crate::context::GatewayContext;
use crate::middleware::{BoxResponse, binary_response, text_response};
use entities::ugc_checkpoints;

fn parse_ugc_uuid(path: &str) -> Option<Uuid> {
    // /checkpoints/<ugcId>
    path.split('/').nth(2).and_then(|s| Uuid::parse_str(s).ok())
}

async fn handle_time_trial(req: Request<HttpBody>, ctx: Arc<GatewayContext>) -> BoxResponse {
    if req.method() != hyper::Method::GET {
        return text_response(StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed");
    }

    let Some(session_header) = req
        .headers()
        .get("x-gatewaysession")
        .and_then(|v| v.to_str().ok())
    else {
        return text_response(StatusCode::UNAUTHORIZED, "Unauthorized");
    };

    if ctx.get_persona_id(session_header).is_none() {
        return text_response(StatusCode::UNAUTHORIZED, "Unauthorized");
    }

    let Some(ugc_id) = parse_ugc_uuid(req.uri().path()) else {
        return text_response(StatusCode::BAD_REQUEST, "Bad Request");
    };

    match ugc_checkpoints::Entity::find_by_id(ugc_id)
        .one(ctx.db())
        .await
    {
        Ok(Some(checkpoint)) => binary_response(checkpoint.data),
        _ => text_response(StatusCode::NOT_FOUND, "Not Found"),
    }
}

#[derive(Clone)]
pub struct CheckpointsRouteLayer {
    pub ctx: Arc<GatewayContext>,
}

impl<S> Layer<S> for CheckpointsRouteLayer {
    type Service = CheckpointsRouteService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CheckpointsRouteService {
            inner,
            ctx: self.ctx.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CheckpointsRouteService<S> {
    inner: S,
    ctx: Arc<GatewayContext>,
}

impl<S, B> Service<Request<HttpBody>> for CheckpointsRouteService<S>
where
    S: Service<Request<HttpBody>, Response = Response<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError> + Send + 'static,
    B: hyper::body::Body<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError> + Send + 'static,
{
    type Response = BoxResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<BoxResponse, BoxError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<HttpBody>) -> Self::Future {
        println!("HTTP {} {}", req.method(), req.uri().path());

        for (key, value) in req.headers() {
            println!("{}: {:?}", key, value);
        }

        if req.uri().path().starts_with("/checkpoints/") {
            let ctx = self.ctx.clone();
            return Box::pin(async move { Ok(handle_time_trial(req, ctx).await) });
        }

        let mut inner = self.inner.clone();
        Box::pin(async move {
            let res = inner.call(req).await.map_err(Into::into)?;
            Ok(res.map(|b| b.map_err(Into::into).boxed_unsync()))
        })
    }
}
