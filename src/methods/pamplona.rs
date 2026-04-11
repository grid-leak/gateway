use std::sync::Arc;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    context::GatewayContext,
    logic::{self, GatewayError, challenge::get_runners_route_data},
    models::{
        customization::{PlayerGhost, PlayerTagResponse, TagData},
        game_data::{
            Entry, LeaderboardResponse, PersonaId, PlayerInfo, ReachThisWrapper, ReplayUrlResponse,
            RunnersRouteData, TimeTrialWrapper, UgcId,
        },
    },
};
use entities::users;

#[rpc(server, namespace = "Pamplona", namespace_separator = ".")]
pub trait Pamplona {
    #[method(name = "getPlayerTags")]
    async fn get_player_tags(
        &self,
        persona_ids: Vec<PersonaId>,
    ) -> RpcResult<Vec<PlayerTagResponse>>;

    #[method(name = "getPlayerTag")]
    async fn get_player_tag(&self, persona_id: PersonaId) -> RpcResult<TagData>;

    #[method(name = "getRunnersRouteData")]
    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
        persona_id: PersonaId,
    ) -> RpcResult<Vec<RunnersRouteData>>;

    #[method(name = "getPlayerGhosts")]
    async fn get_player_ghosts(&self, persona_ids: Vec<PersonaId>) -> RpcResult<Vec<PlayerGhost>>;

    #[method(name = "getPersonaStats")]
    async fn get_persona_stats(
        &self,
        persona_id: PersonaId,
    ) -> RpcResult<serde_json::Map<String, serde_json::Value>>;

    #[method(name = "getLatestPlayed")]
    async fn get_latest_played(&self, persona_id: PersonaId) -> RpcResult<Vec<Entry>>;

    #[method(name = "getPlayerInfo")]
    async fn get_player_info(&self, persona_id: PersonaId) -> RpcResult<PlayerInfo>;

    #[method(name = "getReachThisData")]
    async fn get_reach_this_data(
        &self,
        ugc_ids: Vec<UgcId>,
        data_types: Vec<String>,
        persona_id: PersonaId,
    ) -> RpcResult<Vec<ReachThisWrapper>>;

    #[method(name = "getTimeTrialData")]
    async fn get_time_trial_data(
        &self,
        ugc_ids: Vec<UgcId>,
        data_types: Vec<String>,
        persona_id: PersonaId,
    ) -> RpcResult<Vec<TimeTrialWrapper>>;

    #[method(name = "getTimeTrialLeaderboard")]
    async fn get_time_trial_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
        persona_id: PersonaId,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "getReachThisLeaderboard")]
    async fn get_reach_this_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
        persona_id: PersonaId,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "getReplayURL")]
    async fn get_replay_url(&self, ugc_id: UgcId) -> RpcResult<ReplayUrlResponse>;
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
    async fn get_player_tags(
        &self,
        persona_ids: Vec<PersonaId>,
    ) -> RpcResult<Vec<PlayerTagResponse>> {
        if persona_ids.is_empty() {
            return Ok(vec![]);
        }

        let users = users::Entity::find()
            .filter(users::Column::PersonaId.is_in(persona_ids.into_iter().map(i32::from)))
            .all(self.ctx.db())
            .await
            .map_err(|e| GatewayError::from(e).into_rpc_err())?;

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

    async fn get_player_tag(&self, persona_id: PersonaId) -> RpcResult<TagData> {
        let user = self
            .ctx
            .user(persona_id.into())
            .await
            .map_err(GatewayError::into_rpc_err)?;

        let tag_data: TagData = serde_json::from_value(user.tag_data).map_err(|e| {
            GatewayError::internal(format!("failed to parse tag data: {e}")).into_rpc_err()
        })?;

        Ok(tag_data)
    }

    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
        persona_id: PersonaId,
    ) -> RpcResult<Vec<RunnersRouteData>> {
        get_runners_route_data(&self.ctx, challenge_ids, data_types, persona_id.into())
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_player_ghosts(&self, persona_ids: Vec<PersonaId>) -> RpcResult<Vec<PlayerGhost>> {
        logic::player::get_player_ghosts(
            &self.ctx,
            persona_ids.into_iter().map(i32::from).collect(),
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_persona_stats(
        &self,
        persona_id: PersonaId,
    ) -> RpcResult<serde_json::Map<String, serde_json::Value>> {
        logic::stats::get_persona_stats(&self.ctx, persona_id.into())
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_latest_played(&self, persona_id: PersonaId) -> RpcResult<Vec<Entry>> {
        logic::player::get_latest_played(&self.ctx, persona_id.into())
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_player_info(&self, persona_id: PersonaId) -> RpcResult<PlayerInfo> {
        logic::player::get_player_info(&self.ctx, persona_id.into())
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_reach_this_data(
        &self,
        ugc_ids: Vec<UgcId>,
        data_types: Vec<String>,
        persona_id: PersonaId,
    ) -> RpcResult<Vec<ReachThisWrapper>> {
        let ugc_ids = ugc_ids.into_iter().map(|id| id.id).collect();

        logic::ugc::get_reach_this_data(&self.ctx, ugc_ids, data_types, persona_id.into())
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_time_trial_data(
        &self,
        ugc_ids: Vec<UgcId>,
        data_types: Vec<String>,
        persona_id: PersonaId,
    ) -> RpcResult<Vec<TimeTrialWrapper>> {
        let ugc_ids = ugc_ids.into_iter().map(|id| id.id).collect();

        logic::ugc::get_time_trial_data(&self.ctx, ugc_ids, data_types, persona_id.into())
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_time_trial_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
        persona_id: PersonaId,
    ) -> RpcResult<LeaderboardResponse> {
        logic::leaderboard::get_ugc_leaderboard(
            &self.ctx,
            persona_id.into(),
            ugc_id.id,
            entities::ugc_entries::UgcEntryType::TimeTrial,
            sea_orm::Order::Asc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_reach_this_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
        persona_id: PersonaId,
    ) -> RpcResult<LeaderboardResponse> {
        logic::leaderboard::get_ugc_leaderboard(
            &self.ctx,
            persona_id.into(),
            ugc_id.id,
            entities::ugc_entries::UgcEntryType::ReachThis,
            sea_orm::Order::Asc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_replay_url(&self, ugc_id: UgcId) -> RpcResult<ReplayUrlResponse> {
        let player_ghost = logic::player::get_player_ghosts(&self.ctx, vec![ugc_id.user_id])
            .await
            .map_err(GatewayError::into_rpc_err)?
            .into_iter()
            .next();

        let s3_client = crate::S3_CLIENT.get().expect("S3_CLIENT not initialized");
        let bucket = crate::S3_BUCKET.get().expect("S3_BUCKET not initialized");
        let key = format!("{}/{}", ugc_id.user_id, ugc_id.id);

        let presigned_request = s3_client
            .get_object()
            .bucket(bucket)
            .key(&key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    std::time::Duration::from_secs(3600),
                )
                .map_err(|e| {
                    GatewayError::internal(format!("Failed to build presigning config: {}", e))
                        .into_rpc_err()
                })?,
            )
            .await
            .map_err(|e| {
                GatewayError::internal(format!("Failed to generate presigned url: {}", e))
                    .into_rpc_err()
            })?;

        Ok(ReplayUrlResponse {
            url: Some(presigned_request.uri().to_string()),
            player_ghost,
        })
    }
}
