use sea_orm::EntityTrait;
use std::sync::Arc;
use uuid::Uuid;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;

use crate::{context::GatewayContext, entities::users, models::auth::AuthResponse};

#[rpc(server, namespace = "BeatAuthentication", namespace_separator = ".")]
pub trait BeatAuthentication {
    #[method(name = "viaAuthCode")]
    async fn via_auth_code(&self, auth_code: String) -> RpcResult<AuthResponse>;
}

pub struct BeatAuthenticationImpl {
    ctx: Arc<GatewayContext>,
}

impl BeatAuthenticationImpl {
    pub fn new(ctx: Arc<GatewayContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl BeatAuthenticationServer for BeatAuthenticationImpl {
    async fn via_auth_code(&self, _auth_code: String) -> RpcResult<AuthResponse> {
        // TODO: Implement actual auth code validation logic
        // For now, fetch the first user from the database

        let user_opt = users::Entity::find()
            .one(self.ctx.db())
            .await
            .map_err(|e| {
                jsonrpsee::types::ErrorObject::owned(
                    jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                    e.to_string(),
                    None::<()>,
                )
            })?;

        let persona_id = match user_opt {
            Some(user) => user.persona_id,
            None => {
                return Err(jsonrpsee::types::ErrorObject::owned(
                    -32502,
                    "Authentication failed",
                    None::<()>,
                ));
            }
        };

        let session_id = Uuid::new_v4().to_string();

        self.ctx.register_session(session_id.clone(), persona_id);

        Ok(AuthResponse {
            session_id,
            persona_id,
        })
    }
}
