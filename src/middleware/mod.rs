use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use hyper::{Response, StatusCode, header};
use tower_http::BoxError;

pub mod checkpoints;
pub mod http;
pub mod rpc;
pub mod upload;

pub type BoxBody = UnsyncBoxBody<Bytes, BoxError>;
pub type BoxResponse = Response<BoxBody>;

pub fn text_response(status: StatusCode, body_content: impl Into<Bytes>) -> BoxResponse {
    let body = Full::new(body_content.into())
        .map_err(|e: std::convert::Infallible| -> BoxError { e.into() })
        .boxed_unsync();
    Response::builder().status(status).body(body).unwrap()
}

pub fn binary_response(data: Vec<u8>) -> BoxResponse {
    let body = Full::new(Bytes::from(data))
        .map_err(|e: std::convert::Infallible| -> BoxError { e.into() })
        .boxed_unsync();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(body)
        .unwrap()
}
