use crate::{
    context::GatewayContext,
    logic::{GatewayError, oauth::fetch_discord_user},
    models::auth::AuthResponse,
};
use entities::{accounts, users};
use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;
use uuid::Uuid;

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
    async fn via_auth_code(&self, auth_code: String) -> RpcResult<AuthResponse> {
        let db = self.ctx.db();

        let discord_user = fetch_discord_user(&auth_code).await?;

        let account = accounts::Entity::find()
            .filter(accounts::Column::Provider.eq("discord"))
            .filter(accounts::Column::ProviderUserId.eq(&discord_user.id))
            .one(db)
            .await
            .map_err(GatewayError::from)?;

        let persona_id = match account {
            Some(existing) => {
                let persona_id = existing.persona_id;
                if existing.provider_username != discord_user.username {
                    let mut active: accounts::ActiveModel = existing.into();
                    active.provider_username = ActiveValue::Set(discord_user.username.clone());
                    active.update(db).await.map_err(GatewayError::from)?;
                }
                persona_id
            }
            None => {
                // create a new game user and link the Discord account
                let new_user = users::ActiveModel {
                    persona_id: ActiveValue::NotSet,
                    name: ActiveValue::Set(discord_user.username.clone()),
                    stats: ActiveValue::Set(serde_json::json!({})),
                    division_name: ActiveValue::Set("Copper".to_string()),
                    division_rank: ActiveValue::Set(5),
                    ghost_data: ActiveValue::Set(serde_json::json!({
                        "variation": 244578012,
                        "timestamp": chrono::Utc::now().timestamp(),
                    })),
                    tag_data: ActiveValue::Set(serde_json::json!({
                        "bg":     { "tag": "2556762952" },
                        "detail": { "tag": "1514008114" },
                        "frame":  { "tag": "3049936381" },
                    })),
                };

                let inserted_user = new_user.insert(db).await.map_err(GatewayError::from)?;
                let persona_id = inserted_user.persona_id;

                let new_account = accounts::ActiveModel {
                    id: ActiveValue::NotSet,
                    persona_id: ActiveValue::Set(persona_id),
                    provider: ActiveValue::Set("discord".to_string()),
                    provider_user_id: ActiveValue::Set(discord_user.id.clone()),
                    provider_username: ActiveValue::Set(discord_user.username.clone()),
                };
                new_account.insert(db).await.map_err(GatewayError::from)?;

                persona_id
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
