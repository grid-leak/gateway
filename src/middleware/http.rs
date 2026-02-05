use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderName, HeaderValue};
use hyper::{HeaderMap, Request};
use jsonrpsee::server::HttpBody;
use jsonrpsee::server::HttpRequest;
use sha2::{Digest, Sha256};
use std::env;
use std::sync::OnceLock;
use std::task::{Context, Poll};
use tower::{Layer, Service};

const GATEWAY_SESSION_HEADER: HeaderName = HeaderName::from_static("x-gatewaysession");
static SECRET: OnceLock<Vec<u8>> = OnceLock::new();

pub fn init_secret() -> Result<(), String> {
    let secret_hex =
        std::env::var("GATEWAY_SECRET").map_err(|_| "GATEWAY_SECRET must be set".to_string())?;

    let secret = hex::decode(&secret_hex)
        .map_err(|e| format!("Failed to decode GATEWAY_SECRET as hex: {}", e))?;

    SECRET
        .set(secret)
        .map_err(|_| "SECRET already initialized".to_string())?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum SessionType {
    Identified(String),
    Unknown,
}

fn derive_key(session_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(SECRET.get().expect("SECRET not initialized"));
    hasher.update(env::var("GATEWAY_CLIENT_ID").expect("GATEWAY_CLIENT_ID must be set"));
    hasher.update(session_id.as_bytes());
    let hash = hasher.finalize();

    let mut key = [0u8; 16];
    for i in 0..16 {
        key[i] = hash[i] ^ hash[i + 16];
    }
    key
}

fn decrypt_payload(encrypted_data: &[u8], session_id: &str) -> Result<Vec<u8>, String> {
    use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};

    if encrypted_data.len() < 16 {
        return Err("Encrypted data too short".to_string());
    }

    let key = derive_key(session_id);

    let iv = &encrypted_data[encrypted_data.len() - 16..];
    let payload = &encrypted_data[..encrypted_data.len() - 16];

    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

    let decrypted = Aes128CbcDec::new(&key.into(), iv.into())
        .decrypt_padded_vec_mut::<Pkcs7>(payload)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    Ok(decrypted)
}

fn parse_session(headers: &HeaderMap<HeaderValue>) -> SessionType {
    match headers.get(GATEWAY_SESSION_HEADER).cloned() {
        Some(session) => SessionType::Identified(session.to_str().unwrap().to_string()),
        None => SessionType::Unknown,
    }
}

#[derive(Clone)]
pub struct HttpMiddlewareLayer;

impl HttpMiddlewareLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for HttpMiddlewareLayer {
    type Service = HttpMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HttpMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct HttpMiddleware<S> {
    inner: S,
}

impl<S> Service<HttpRequest<HttpBody>> for HttpMiddleware<S>
where
    S: Service<HttpRequest<Full<Bytes>>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: HttpRequest<HttpBody>) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let session = parse_session(req.headers());

            // TODO: pass info about encryption to RPC middleware
            // so it doesn't let unencrypted requests that require
            // encryption to be processed
            let should_decrypt = req
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|v| v == "application/x-encrypted")
                .unwrap_or(false);

            let (mut parts, body) = req.into_parts();

            // Collect the body
            let body_bytes = match body.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => {
                    // Return error by creating a request with empty body
                    let mut new_req = Request::from_parts(parts, Full::new(Bytes::new()));
                    new_req.extensions_mut().insert(session);
                    return inner.call(new_req).await;
                }
            };

            let final_body = if should_decrypt {
                // Get the session ID for decryption
                let session_id = match &session {
                    SessionType::Identified(id) => id.clone(),
                    SessionType::Unknown => {
                        tracing::warn!("Encrypted request without session ID");
                        // Cannot decrypt without session ID
                        let mut new_req = Request::from_parts(parts, Full::new(body_bytes));
                        new_req.extensions_mut().insert(session);
                        return inner.call(new_req).await;
                    }
                };

                // Decrypt the payload
                match decrypt_payload(&body_bytes, &session_id) {
                    Ok(decrypted) => {
                        // Update content-type to application/json
                        // so jsonrpsee can properly handle it
                        parts
                            .headers
                            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

                        // Update content-length
                        parts.headers.insert(
                            CONTENT_LENGTH,
                            HeaderValue::from_str(&decrypted.len().to_string()).unwrap(),
                        );

                        Bytes::from(decrypted)
                    }
                    Err(_) => {
                        // Return original body on decryption failure
                        body_bytes
                    }
                }
            } else {
                // Not encrypted
                body_bytes
            };

            // Create new request with the final body
            let mut new_req = Request::from_parts(parts, Full::new(final_body));

            // Insert session into extensions for RPC middleware
            new_req.extensions_mut().insert(session);

            inner.call(new_req).await
        })
    }
}
