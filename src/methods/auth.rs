use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;
use uuid::Uuid;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;

use crate::{
    context::GatewayContext, entities::users, methods::map_err, models::auth::AuthResponse,
};

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

        let user_opt = users::Entity::find()
            .filter(users::Column::PersonaId.eq(1011786733))
            .one(self.ctx.db())
            .await
            .map_err(map_err)?;

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
