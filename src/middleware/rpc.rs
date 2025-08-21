use jsonrpsee::{
    MethodResponse,
    core::middleware::{ResponseFuture, RpcServiceT},
    types::ErrorObject,
};
use tower::Layer;

use crate::middleware::http::SessionType;

#[derive(Clone)]
pub struct RpcMiddlewareLayer;

impl<S> Layer<S> for RpcMiddlewareLayer
where
    S: RpcServiceT + Send + Sync + Clone + 'static,
{
    type Service = RpcMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RpcMiddleware(inner)
    }
}

#[derive(Clone)]
pub struct RpcMiddleware<S>(S);
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
        request: jsonrpsee::types::Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        if request.method_name().starts_with("PamplonaAuthenticated") {
            let session = request.extensions.get::<SessionType>().unwrap();

            match session {
                SessionType::Identified(_session_id) => {
                    // TODO: check session_id against a list of authorized sessions
                    ResponseFuture::future(self.0.call(request))
                }
                SessionType::Unknown => ResponseFuture::ready(MethodResponse::error(
                    request.id,
                    // TODO: implement custom error codes from the game
                    // and find the one that is responsible for this
                    // so the game can show the correct error message
                    // or re-authenticate if possible
                    ErrorObject::borrowed(-32000, "Unauthorized.", None),
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
