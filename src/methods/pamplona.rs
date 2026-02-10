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
        customization::{PlayerGhost, PlayerTagResponse, TagData},
        game_data::{Entry, PlayerInfo, ReachThisWrapper, RunnersRouteData, UgcId},
    },
};

#[rpc(server, namespace = "Pamplona", namespace_separator = ".")]
pub trait Pamplona {
    #[method(name = "getPlayerTags")]
    async fn get_player_tags(&self, persona_ids: Vec<i32>) -> RpcResult<Vec<PlayerTagResponse>>;

    #[method(name = "getPlayerTag")]
    async fn get_player_tag(&self, persona_id: i32) -> RpcResult<TagData>;

    #[method(name = "getRunnersRouteData")]
    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
        persona_id: i32,
    ) -> RpcResult<Vec<RunnersRouteData>>;

    #[method(name = "getPlayerGhosts")]
    async fn get_player_ghosts(&self, persona_ids: Vec<i32>) -> RpcResult<Vec<PlayerGhost>>;

    #[method(name = "getPersonaStats")]
    async fn get_persona_stats(
        &self,
        persona_id: i32,
    ) -> RpcResult<serde_json::Map<String, serde_json::Value>>;

    #[method(name = "getLatestPlayed")]
    async fn get_latest_played(&self, persona_id: i32) -> RpcResult<Vec<Entry>>;

    #[method(name = "getPlayerInfo")]
    async fn get_player_info(&self, persona_id: i32) -> RpcResult<PlayerInfo>;

    #[method(name = "getReachThisData")]
    async fn get_reach_this_data(
        &self,
        ugc_ids: Vec<UgcId>,
        data_types: Vec<String>,
        persona_id: i32,
    ) -> RpcResult<Vec<ReachThisWrapper>>;
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
    async fn get_player_tags(&self, persona_ids: Vec<i32>) -> RpcResult<Vec<PlayerTagResponse>> {
        if persona_ids.is_empty() {
            return Ok(vec![]);
        }

        let users = users::Entity::find()
            .filter(users::Column::PersonaId.is_in(persona_ids))
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

    async fn get_player_tag(&self, persona_id: i32) -> RpcResult<TagData> {
        let user = users::Entity::find_by_id(persona_id)
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
        persona_id: i32,
    ) -> RpcResult<Vec<RunnersRouteData>> {
        get_runners_route_data(&self.ctx, challenge_ids, data_types, persona_id)
            .await
            .map_err(map_err)
    }

    async fn get_player_ghosts(&self, persona_ids: Vec<i32>) -> RpcResult<Vec<PlayerGhost>> {
        logic::player::get_player_ghosts(&self.ctx, persona_ids)
            .await
            .map_err(map_err)
    }

    async fn get_persona_stats(
        &self,
        persona_id: i32,
    ) -> RpcResult<serde_json::Map<String, serde_json::Value>> {
        logic::stats::get_persona_stats(&self.ctx, persona_id).await
    }

    async fn get_latest_played(&self, persona_id: i32) -> RpcResult<Vec<Entry>> {
        logic::player::get_latest_played(&self.ctx, persona_id)
            .await
            .map_err(map_err)
    }

    async fn get_player_info(&self, persona_id: i32) -> RpcResult<PlayerInfo> {
        logic::player::get_player_info(&self.ctx, persona_id)
            .await
            .map_err(map_err)
    }

    async fn get_reach_this_data(
        &self,
        ugc_ids: Vec<UgcId>,
        data_types: Vec<String>,
        persona_id: i32,
    ) -> RpcResult<Vec<ReachThisWrapper>> {
        let ugc_ids = ugc_ids.into_iter().map(|id| id.id).collect();

        logic::ugc::get_reach_this_data(&self.ctx, ugc_ids, data_types, persona_id)
            .await
            .map_err(map_err)
    }
}
