use std::sync::Arc;
use uuid::Uuid;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;

use crate::{context::GatewayContext, models::auth::AuthResponse};

// TODO: web companion uses "BeatAuthentication", while the game uses "Authentication"
#[rpc(server, namespace = "Authentication", namespace_separator = ".")]
pub trait Authentication {
    #[method(name = "viaAuthCode")]
    async fn via_auth_code(&self, auth_code: String) -> RpcResult<AuthResponse>;
}

pub struct AuthenticationImpl {
    ctx: Arc<GatewayContext>,
}

impl AuthenticationImpl {
    pub fn new(ctx: Arc<GatewayContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl AuthenticationServer for AuthenticationImpl {
    async fn via_auth_code(&self, _auth_code: String) -> RpcResult<AuthResponse> {
        // TODO: Implement actual auth code validation logic
        // For now, fetch the first user from the database

        let user = self.ctx.user(1011786733).await?;
        let persona_id = user.persona_id;

        let session_id = Uuid::new_v4().to_string();

        self.ctx.register_session(session_id.clone(), persona_id);

        Ok(AuthResponse {
            session_id,
            persona_id,
        })
    }
}
