use crate::{
    context::GatewayContext,
    logic,
    methods::map_err,
    models::{
        customization::GhostDataInput,
        game_data::{
            Entry, HackableBillboardLeader, InitialGameDataResponse, Inventory, Item, Kit,
            OverviewReachThisLeaderboardResponse, PlayerUgcLimits, ReachThisWrapper,
            RunnersRouteData, UgcId, UgcMeta,
        },
        ugc::CreateReachThisMeta,
    },
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
};
use jsonrpsee_proc_macros::rpc;
use std::collections::HashMap;
use std::sync::Arc;

type UserStats = serde_json::Map<String, serde_json::Value>;

#[rpc(server, namespace = "PamplonaAuthenticated", namespace_separator = ".")]
pub trait PamplonaAuthenticated {
    #[method(name = "getInitialGameData", with_extensions)]
    async fn get_initial_game_data(
        &self,
        level_ids: Vec<u32>,
    ) -> RpcResult<InitialGameDataResponse>;

    #[method(name = "getInventory", with_extensions)]
    async fn get_inventory(&self) -> RpcResult<Inventory>;

    #[method(name = "getRunnersRouteData", with_extensions)]
    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
    ) -> RpcResult<Vec<RunnersRouteData>>;

    #[method(name = "getHackableBillboardFriendsLeaders", with_extensions)]
    async fn get_hackable_billboard_friends_leaders(
        &self,
        challenge_ids: Vec<String>,
    ) -> RpcResult<HashMap<String, Option<HackableBillboardLeader>>>;

    #[method(name = "setPlayerGhost", with_extensions)]
    async fn set_player_ghost(&self, ghost_data: GhostDataInput) -> RpcResult<String>;

    #[method(name = "setPlayerTag", with_extensions)]
    async fn set_player_tag(
        &self,
        tag_data: crate::models::customization::TagData,
    ) -> RpcResult<String>;

    #[method(name = "grantKit", with_extensions)]
    async fn grant_kit(&self, id: String) -> RpcResult<Kit>;

    #[method(name = "openKit", with_extensions)]
    async fn open_kit(&self, id: String) -> RpcResult<Vec<Item>>;

    #[method(name = "revokeKit", with_extensions)]
    async fn revoke_kit(&self, id: String) -> RpcResult<Vec<Item>>;

    #[method(name = "updatePersonaStats", with_extensions)]
    async fn update_persona_stats(&self, stats: UserStats) -> RpcResult<String>;

    #[method(name = "finishHackableBillboard", with_extensions)]
    async fn finish_hackable_billboard(
        &self,
        challenge_id: String,
        main_stat: i32,
        extra_stats: serde_json::Value,
    ) -> RpcResult<String>;

    #[method(name = "getPersonaStats", with_extensions)]
    async fn get_persona_stats(&self) -> RpcResult<UserStats>;

    #[method(name = "getLatestPlayed", with_extensions)]
    async fn get_latest_played(&self) -> RpcResult<Vec<Entry>>;

    #[method(name = "getPlayerUgcLimits", with_extensions)]
    async fn get_player_ugc_limits(&self) -> RpcResult<PlayerUgcLimits>;

    #[method(name = "createReachThis", with_extensions)]
    async fn create_reach_this(
        &self,
        data: String,
        meta: CreateReachThisMeta,
    ) -> RpcResult<UgcMeta>;

    #[method(name = "finishReachThis", with_extensions)]
    async fn finish_reach_this(&self, ugc_id: UgcId) -> RpcResult<ReachThisWrapper>;

    #[method(name = "getOverviewReachThisLeaderboard", with_extensions)]
    async fn get_overview_reach_this_leaderboard(
        &self,
        ugc_id: UgcId,
        radius: Option<i32>,
    ) -> RpcResult<OverviewReachThisLeaderboardResponse>;
}

pub struct PamplonaAuthenticatedImpl {
    ctx: Arc<GatewayContext>,
}

impl PamplonaAuthenticatedImpl {
    pub fn new(ctx: Arc<GatewayContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl PamplonaAuthenticatedServer for PamplonaAuthenticatedImpl {
    async fn get_initial_game_data(
        &self,
        extensions: &Extensions,
        level_ids: Vec<u32>,
    ) -> RpcResult<InitialGameDataResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::ugc::get_initial_game_data(&self.ctx, level_ids[0], persona_id).await
    }

    async fn get_inventory(&self, extensions: &Extensions) -> RpcResult<Inventory> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::get_inventory(&self.ctx, persona_id)
            .await
            .map_err(map_err)
    }

    async fn get_runners_route_data(
        &self,
        extensions: &Extensions,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
    ) -> RpcResult<Vec<RunnersRouteData>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::challenge::get_runners_route_data(&self.ctx, challenge_ids, data_types, persona_id)
            .await
            .map_err(map_err)
    }

    async fn get_hackable_billboard_friends_leaders(
        &self,
        _extensions: &Extensions,
        challenge_ids: Vec<String>,
    ) -> RpcResult<HashMap<String, Option<HackableBillboardLeader>>> {
        logic::challenge::get_hackable_billboard_friends_leaders(&self.ctx, challenge_ids)
            .await
            .map_err(map_err)
    }

    async fn set_player_ghost(
        &self,
        extensions: &Extensions,
        ghost_data: GhostDataInput,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::player::set_player_ghost(&self.ctx, persona_id, ghost_data)
            .await
            .map_err(map_err)?;

        Ok("success".to_string())
    }

    async fn set_player_tag(
        &self,
        extensions: &Extensions,
        tag_data: crate::models::customization::TagData,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::player::set_player_tag(&self.ctx, persona_id, tag_data)
            .await
            .map_err(map_err)?;

        Ok("success".to_string())
    }

    async fn grant_kit(&self, extensions: &Extensions, id: String) -> RpcResult<Kit> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::grant_kit(&self.ctx, persona_id, &id)
            .await
            .map_err(map_err)
    }

    async fn open_kit(&self, extensions: &Extensions, id: String) -> RpcResult<Vec<Item>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::open_kit(&self.ctx, persona_id, &id)
            .await
            .map_err(map_err)
    }

    async fn revoke_kit(&self, extensions: &Extensions, id: String) -> RpcResult<Vec<Item>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::revoke_kit(&self.ctx, persona_id, &id)
            .await
            .map_err(map_err)
    }

    async fn update_persona_stats(
        &self,
        extensions: &Extensions,
        stats: UserStats,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::stats::update_persona_stats(&self.ctx, persona_id, stats).await
    }

    async fn finish_hackable_billboard(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        main_stat: i32,
        extra_stats: serde_json::Value,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::challenge::finish_hackable_billboard(
            &self.ctx,
            persona_id,
            challenge_id,
            main_stat,
            extra_stats,
        )
        .await
        .map_err(map_err)?;

        Ok("success".to_string())
    }

    async fn get_persona_stats(&self, extensions: &Extensions) -> RpcResult<UserStats> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::stats::get_persona_stats(&self.ctx, persona_id).await
    }

    async fn get_latest_played(&self, extensions: &Extensions) -> RpcResult<Vec<Entry>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::player::get_latest_played(&self.ctx, persona_id)
            .await
            .map_err(map_err)
    }

    async fn get_player_ugc_limits(&self, extensions: &Extensions) -> RpcResult<PlayerUgcLimits> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::player::get_player_ugc_limits(&self.ctx, persona_id)
            .await
            .map_err(map_err)
    }

    async fn create_reach_this(
        &self,
        extensions: &Extensions,
        _data: String,
        meta: CreateReachThisMeta,
    ) -> RpcResult<UgcMeta> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::ugc::create_reach_this(&self.ctx, persona_id, meta)
            .await
            .map_err(map_err)
    }

    async fn finish_reach_this(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
    ) -> RpcResult<ReachThisWrapper> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::ugc::finish_reach_this(&self.ctx, persona_id, ugc_id.id, ugc_id.user_id)
            .await
            .map_err(map_err)
    }

    async fn get_overview_reach_this_leaderboard(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        radius: Option<i32>,
    ) -> RpcResult<OverviewReachThisLeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::ugc::get_overview_reach_this_leaderboard(&self.ctx, persona_id, ugc_id.id, radius)
            .await
            .map_err(map_err)
    }
}
