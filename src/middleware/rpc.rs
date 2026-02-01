use std::future::Future;
use std::sync::Arc;

use jsonrpsee::{
    MethodResponse,
    core::middleware::{ResponseFuture, RpcServiceT},
    types::ErrorObject,
};
use tower::Layer;

use crate::{context::GatewayContext, middleware::http::SessionType};

#[derive(Clone)]
pub struct RpcMiddlewareLayer {
    ctx: Arc<GatewayContext>,
}

impl RpcMiddlewareLayer {
    pub fn new(ctx: Arc<GatewayContext>) -> Self {
        Self { ctx }
    }
}

impl<S> Layer<S> for RpcMiddlewareLayer
where
    S: RpcServiceT + Send + Sync + Clone + 'static,
{
    type Service = RpcMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RpcMiddleware(inner, self.ctx.clone())
    }
}

#[derive(Clone)]
pub struct RpcMiddleware<S>(S, Arc<GatewayContext>);
impl<S> RpcServiceT for RpcMiddleware<S>
where
    S: RpcServiceT<
            MethodResponse = MethodResponse,
            BatchResponse = MethodResponse,
            NotificationResponse = MethodResponse,
        > + Send
        + Sync
        + Clone
        + 'static,
{
    type MethodResponse = S::MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn batch<'a>(
        &self,
        requests: jsonrpsee::core::middleware::Batch<'a>,
    ) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        self.0.batch(requests)
    }

    fn call<'a>(
        &self,
        mut request: jsonrpsee::types::Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        println!("Received {}", request.method_name());

        if request.method_name().starts_with("PamplonaAuthenticated") {
            let session_type = request.extensions.get::<SessionType>().unwrap();
            let ctx = &self.1;

            match session_type {
                SessionType::Identified(session_id) => {
                    if let Some(persona_id) = ctx.get_persona_id(session_id) {
                        request.extensions.insert(persona_id);
                        ResponseFuture::future(self.0.call(request))
                    } else {
                        ResponseFuture::ready(MethodResponse::error(
                            request.id,
                            ErrorObject::borrowed(-32501, "Invalid Params: no valid session", None),
                        ))
                    }
                }
                SessionType::Unknown => ResponseFuture::ready(MethodResponse::error(
                    request.id,
                    ErrorObject::borrowed(-32501, "Invalid Params: no valid session", None),
                )),
            }
        } else {
            ResponseFuture::future(self.0.call(request))
        }
    }

    fn notification<'a>(
        &self,
        n: jsonrpsee::core::middleware::Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        self.0.notification(n)
    }
}
