use std::sync::Arc;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    context::GatewayContext,
    entities::users,
    logic::{self, challenge::get_runners_route_data},
    methods::map_err,
    models::{
        challenge::RunnersRouteDataResponse,
        customization::{PlayerGhost, PlayerTagResponse, TagData},
    },
};

#[rpc(server, namespace = "Pamplona", namespace_separator = ".")]
pub trait Pamplona {
    #[method(name = "getPlayerTags")]
    async fn get_player_tags(&self, persona_ids: Vec<String>) -> RpcResult<Vec<PlayerTagResponse>>;

    #[method(name = "getPlayerTag")]
    async fn get_player_tag(&self, persona_id: String) -> RpcResult<TagData>;

    #[method(name = "getRunnersRouteData")]
    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
        persona_id: String,
    ) -> RpcResult<Vec<RunnersRouteDataResponse>>;

    #[method(name = "getPlayerGhosts")]
    async fn get_player_ghosts(&self, persona_ids: Vec<i32>) -> RpcResult<Vec<PlayerGhost>>;

    #[method(name = "getPersonaStats")]
    async fn get_persona_stats(
        &self,
        persona_id: i32,
    ) -> RpcResult<serde_json::Map<String, serde_json::Value>>;
}

pub struct PamplonaImpl {
    ctx: Arc<GatewayContext>,
}

impl PamplonaImpl {
    pub fn new(ctx: Arc<GatewayContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl PamplonaServer for PamplonaImpl {
    async fn get_player_tags(&self, persona_ids: Vec<String>) -> RpcResult<Vec<PlayerTagResponse>> {
        let persona_ids_int: Vec<i32> = persona_ids
            .iter()
            .filter_map(|id| id.parse::<i32>().ok())
            .collect();

        if persona_ids_int.is_empty() {
            return Ok(vec![]);
        }

        let users = users::Entity::find()
            .filter(users::Column::PersonaId.is_in(persona_ids_int))
            .all(self.ctx.db())
            .await
            .map_err(map_err)?;

        let response = users
            .into_iter()
            .filter_map(|user| {
                let tag_data: TagData = serde_json::from_value(user.tag_data).ok()?;

                Some(PlayerTagResponse {
                    persona_id: user.persona_id.to_string(),
                    tag_data,
                })
            })
            .collect();

        Ok(response)
    }

    async fn get_player_tag(&self, persona_id: String) -> RpcResult<TagData> {
        let persona_id_int = persona_id.parse::<i32>().map_err(|_| {
            jsonrpsee::types::ErrorObject::owned(
                jsonrpsee::types::error::INVALID_PARAMS_CODE,
                "Invalid persona_id",
                None::<()>,
            )
        })?;

        let user = users::Entity::find_by_id(persona_id_int)
            .one(self.ctx.db())
            .await
            .map_err(map_err)?
            .ok_or_else(|| {
                jsonrpsee::types::ErrorObject::owned(
                    jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                    "User not found",
                    None::<()>,
                )
            })?;

        let tag_data: TagData = serde_json::from_value(user.tag_data).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                format!("Failed to parse tag data: {}", e),
                None::<()>,
            )
        })?;

        Ok(tag_data)
    }

    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
        persona_id: String,
    ) -> RpcResult<Vec<RunnersRouteDataResponse>> {
        let pid = persona_id.parse::<i32>().ok();

        if pid.is_none() {
            return Err(jsonrpsee::types::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                "Invalid persona id",
                None::<()>,
            ));
        }

        let pid = pid.unwrap();

        get_runners_route_data(&self.ctx, challenge_ids, data_types, pid)
            .await
            .map_err(map_err)
    }

    async fn get_player_ghosts(&self, persona_ids: Vec<i32>) -> RpcResult<Vec<PlayerGhost>> {
        logic::customization::get_player_ghosts(&self.ctx, persona_ids)
            .await
            .map_err(map_err)
    }

    async fn get_persona_stats(
        &self,
        persona_id: i32,
    ) -> RpcResult<serde_json::Map<String, serde_json::Value>> {
        logic::stats::get_persona_stats(&self.ctx, persona_id).await
    }
}
