use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Request, Response, StatusCode, header};
use jsonrpsee::server::HttpBody;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tower_http::BoxError;
use uuid::Uuid;

use crate::context::GatewayContext;
use crate::middleware::{BoxResponse, text_response};

async fn handle_upload(req: Request<HttpBody>, ctx: Arc<GatewayContext>) -> BoxResponse {
    if req.method() != hyper::Method::POST {
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

    let Some(content_length_str) = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
    else {
        return text_response(StatusCode::LENGTH_REQUIRED, "Length Required");
    };

    const MAX_UPLOAD_BYTES: i64 = 2048 * 1024;

    let content_length: i64 = match content_length_str.parse() {
        Ok(len) if len > 0 && len <= MAX_UPLOAD_BYTES => len,
        Ok(len) if len > MAX_UPLOAD_BYTES => {
            return text_response(StatusCode::PAYLOAD_TOO_LARGE, "Payload Too Large");
        }
        _ => return text_response(StatusCode::LENGTH_REQUIRED, "Invalid Length"),
    };

    let ticket_uuid = Uuid::new_v4();
    let s3_key = format!("tickets/{}", ticket_uuid);

    let body_bytes = match req.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => return text_response(StatusCode::BAD_REQUEST, "Bad Request Body"),
    };

    let body_stream = aws_sdk_s3::primitives::ByteStream::from(body_bytes);

    match crate::S3_CLIENT
        .get()
        .expect("S3_CLIENT not initialized")
        .put_object()
        .bucket(crate::S3_BUCKET.get().expect("S3_BUCKET not initialized"))
        .key(s3_key)
        .content_length(content_length)
        .body(body_stream)
        .send()
        .await
    {
        Ok(_) => text_response(StatusCode::OK, ticket_uuid.to_string()),
        Err(e) => {
            tracing::error!("S3 upload failed: {:?}", e);
            text_response(StatusCode::BAD_GATEWAY, "Upload Failed")
        }
    }
}

#[derive(Clone)]
pub struct UploadRouteLayer {
    pub ctx: Arc<GatewayContext>,
}

impl<S> Layer<S> for UploadRouteLayer {
    type Service = UploadRouteService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        UploadRouteService {
            inner,
            ctx: self.ctx.clone(),
        }
    }
}

#[derive(Clone)]
pub struct UploadRouteService<S> {
    inner: S,
    ctx: Arc<GatewayContext>,
}

impl<S, B> Service<Request<HttpBody>> for UploadRouteService<S>
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
        if req.uri().path() == "/upload" {
            let ctx = self.ctx.clone();
            return Box::pin(async move { Ok(handle_upload(req, ctx).await) });
        }

        let mut inner = self.inner.clone();
        Box::pin(async move {
            let res = inner.call(req).await.map_err(Into::into)?;
            Ok(res.map(|b| b.map_err(Into::into).boxed_unsync()))
        })
    }
}
