use hyper::HeaderMap;
use hyper::header::HeaderName;
use hyper::header::HeaderValue;
use jsonrpsee::server::HttpBody;
use jsonrpsee::server::HttpRequest;
use tower::Layer;
use tower::Service;

pub const GATEWAY_SESSION_HEADER: HeaderName = HeaderName::from_static("x-gatewaysession");

#[derive(Debug, Clone)]
pub enum SessionType {
    Identified(String),
    Unknown,
}

#[derive(Clone)]
pub struct HttpMiddlewareLayer {}

impl HttpMiddlewareLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for HttpMiddlewareLayer {
    type Service = HttpMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        HttpMiddleware { inner: service }
    }
}

#[derive(Clone)]
pub struct HttpMiddleware<S> {
    inner: S,
}

impl<S> Service<HttpRequest<HttpBody>> for HttpMiddleware<S>
where
    S: Service<HttpRequest<HttpBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: HttpRequest<HttpBody>) -> Self::Future {
        // Extract the session header and put it in extensions
        // where it can be accessed by the RPC middleware
        let session = parse_session(request.headers());
        request.extensions_mut().insert(session);

        self.inner.call(request)
    }
}

fn parse_session(headers: &HeaderMap<HeaderValue>) -> SessionType {
    match headers.get(GATEWAY_SESSION_HEADER).cloned() {
        Some(session) => SessionType::Identified(session.to_str().unwrap().to_string()),
        None => SessionType::Unknown,
    }
}
