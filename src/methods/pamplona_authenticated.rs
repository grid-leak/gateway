use crate::{
    context::GatewayContext,
    logic::{self, GatewayError},
    models::{
        customization::GhostDataInput,
        game_data::{
            Bookmarks, Division, Entry, HackableBillboardLeader, InitialGameDataResponse,
            Inventory, Item, Kit, LeaderboardResponse, OverviewLeaderboardResponse,
            PlayerUgcLimits, RunnersRouteData, UgcId, UgcMeta,
        },
        ugc::{CreateReachThisMeta, CreateTimeTrialMeta},
    },
};
use entities::{challenge_entries::ChallengeEntryType, ugc, ugc_entries::UgcEntryType};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
};
use jsonrpsee_proc_macros::rpc;
use sea_orm::{EntityTrait, Order};
use std::collections::HashMap;
use std::sync::Arc;

type UserStats = serde_json::Map<String, serde_json::Value>;

#[rpc(server, namespace = "PamplonaAuthenticated", namespace_separator = ".")]
pub trait PamplonaAuthenticated {
    #[method(name = "getInitialGameData", with_extensions)]
    async fn get_initial_game_data(
        &self,
        level_ids: Vec<i64>,
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

    #[method(name = "getBookmarks", with_extensions)]
    async fn get_bookmarks(&self, level_ids: Vec<i32>) -> RpcResult<Bookmarks>;

    #[method(name = "addUGCBookmark", with_extensions)]
    async fn add_ugc_bookmark(&self, ugc_type: String, ugc_id: UgcId) -> RpcResult<String>;

    #[method(name = "removeUGCBookmark", with_extensions)]
    async fn remove_ugc_bookmark(&self, ugc_type: String, ugc_id: UgcId) -> RpcResult<String>;

    #[method(name = "addChallengeBookmark", with_extensions)]
    async fn add_challenge_bookmark(
        &self,
        challenge_id: String,
        challenge_type: String,
    ) -> RpcResult<String>;

    #[method(name = "removeChallengeBookmark", with_extensions)]
    async fn remove_challenge_bookmark(
        &self,
        challenge_id: String,
        challenge_type: String,
    ) -> RpcResult<String>;

    #[method(name = "getPlayerUGCLimits", with_extensions)]
    async fn get_player_ugc_limits(&self) -> RpcResult<PlayerUgcLimits>;

    #[method(name = "createReachThis", with_extensions)]
    async fn create_reach_this(
        &self,
        data: String,
        meta: CreateReachThisMeta,
    ) -> RpcResult<UgcMeta>;

    #[method(name = "createTimeTrial", with_extensions)]
    async fn create_time_trial(
        &self,
        data: String,
        meta: CreateTimeTrialMeta,
    ) -> RpcResult<UgcMeta>;

    #[method(name = "finishReachThis", with_extensions)]
    async fn finish_reach_this(&self, ugc_id: UgcId) -> RpcResult<String>;

    #[method(name = "getOverviewReachThisLeaderboard", with_extensions)]
    async fn get_overview_reach_this_leaderboard(
        &self,
        ugc_id: UgcId,
        radius: Option<i32>,
    ) -> RpcResult<OverviewLeaderboardResponse>;

    #[method(name = "getOverviewTimeTrialLeaderboard", with_extensions)]
    async fn get_overview_time_trial_leaderboard(
        &self,
        ugc_id: UgcId,
        radius: Option<i32>,
    ) -> RpcResult<OverviewLeaderboardResponse>;

    #[method(name = "getHackableBillboardFriendsLeaderboard", with_extensions)]
    async fn get_hackable_billboard_friends_leaderboard(
        &self,
        challenge_id: String,
        offset: Option<i64>,
        count: i64,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "getOverviewRunnersRouteLeaderboard", with_extensions)]
    async fn get_overview_runners_route_leaderboard(
        &self,
        challenge_id: String,
        radius: Option<i32>,
    ) -> RpcResult<OverviewLeaderboardResponse>;

    #[method(name = "getRunnersRouteFriendsLeaderboard", with_extensions)]
    async fn get_runners_route_friends_leaderboard(
        &self,
        challenge_id: String,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "getRunnersRouteLeaderboard", with_extensions)]
    async fn get_runners_route_leaderboard(
        &self,
        challenge_id: String,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "finishRunnersRoute", with_extensions)]
    async fn finish_runners_route(
        &self,
        challenge_id: String,
        main_stat: i32,
        extra_stats: serde_json::Value,
        run_id: i32,
    ) -> RpcResult<Division>;

    #[method(name = "startTimeTrial", with_extensions)]
    async fn start_time_trial(&self, ugc_id: UgcId) -> RpcResult<String>;

    #[method(name = "cancelTimeTrial", with_extensions)]
    async fn cancel_time_trial(&self, ugc_id: UgcId) -> RpcResult<String>;

    #[method(name = "finishTimeTrial", with_extensions)]
    async fn finish_time_trial(
        &self,
        ugc_id: UgcId,
        finish_time: i64,
        replay_upload_ticket: String,
        extra_stats: serde_json::Value,
        split_times: Vec<i64>,
    ) -> RpcResult<String>;

    #[method(name = "getTimeTrialLeaderboard", with_extensions)]
    async fn get_time_trial_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "getReachThisLeaderboard", with_extensions)]
    async fn get_reach_this_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "getTimeTrialFriendsLeaderboard", with_extensions)]
    async fn get_time_trial_friends_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse>;

    #[method(name = "getReachThisFriendsLeaderboard", with_extensions)]
    async fn get_reach_this_friends_leaderboard(
        &self,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse>;
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
        level_ids: Vec<i64>,
    ) -> RpcResult<InitialGameDataResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::ugc::get_initial_game_data(&self.ctx, level_ids[0], persona_id)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_inventory(&self, extensions: &Extensions) -> RpcResult<Inventory> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::get_inventory(&self.ctx, persona_id)
            .await
            .map_err(GatewayError::into_rpc_err)
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
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_hackable_billboard_friends_leaders(
        &self,
        _extensions: &Extensions,
        challenge_ids: Vec<String>,
    ) -> RpcResult<HashMap<String, Option<HackableBillboardLeader>>> {
        logic::challenge::get_hackable_billboard_friends_leaders(&self.ctx, challenge_ids)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn set_player_ghost(
        &self,
        extensions: &Extensions,
        ghost_data: GhostDataInput,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::player::set_player_ghost(&self.ctx, persona_id, ghost_data)
            .await
            .map_err(GatewayError::into_rpc_err)?;

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
            .map_err(GatewayError::into_rpc_err)?;

        Ok("success".to_string())
    }

    async fn grant_kit(&self, extensions: &Extensions, id: String) -> RpcResult<Kit> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::grant_kit(&self.ctx, persona_id, &id)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn open_kit(&self, extensions: &Extensions, id: String) -> RpcResult<Vec<Item>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::open_kit(&self.ctx, persona_id, &id)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn revoke_kit(&self, extensions: &Extensions, id: String) -> RpcResult<Vec<Item>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::inventory::revoke_kit(&self.ctx, persona_id, &id)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn update_persona_stats(
        &self,
        extensions: &Extensions,
        stats: UserStats,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::stats::update_persona_stats(&self.ctx, persona_id, stats)
            .await
            .map_err(GatewayError::into_rpc_err)
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
        .map_err(GatewayError::into_rpc_err)?;

        Ok("success".to_string())
    }

    async fn get_persona_stats(&self, extensions: &Extensions) -> RpcResult<UserStats> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::stats::get_persona_stats(&self.ctx, persona_id)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_latest_played(&self, extensions: &Extensions) -> RpcResult<Vec<Entry>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::player::get_latest_played(&self.ctx, persona_id)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn get_bookmarks(
        &self,
        extensions: &Extensions,
        _level_ids: Vec<i32>,
    ) -> RpcResult<Bookmarks> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::bookmark::get_bookmarks(&self.ctx, persona_id)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn add_ugc_bookmark(
        &self,
        extensions: &Extensions,
        _ugc_type: String,
        ugc_id: UgcId,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        let ugc_uuid = uuid::Uuid::parse_str(&ugc_id.id)
            .map_err(|_| GatewayError::invalid_params("invalid UGC UUID").into_rpc_err())?;
        logic::bookmark::add_ugc_bookmark(&self.ctx, persona_id, ugc_uuid)
            .await
            .map_err(GatewayError::into_rpc_err)?;
        Ok("success".to_string())
    }

    async fn remove_ugc_bookmark(
        &self,
        extensions: &Extensions,
        _ugc_type: String,
        ugc_id: UgcId,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        let ugc_uuid = uuid::Uuid::parse_str(&ugc_id.id)
            .map_err(|_| GatewayError::invalid_params("invalid UGC UUID").into_rpc_err())?;
        logic::bookmark::remove_ugc_bookmark(&self.ctx, persona_id, ugc_uuid)
            .await
            .map_err(GatewayError::into_rpc_err)?;
        Ok("success".to_string())
    }

    async fn add_challenge_bookmark(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        challenge_type: String,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::bookmark::add_challenge_bookmark(
            &self.ctx,
            persona_id,
            challenge_id,
            challenge_type,
        )
        .await
        .map_err(GatewayError::into_rpc_err)?;
        Ok("success".to_string())
    }

    async fn remove_challenge_bookmark(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        _challenge_type: String,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::bookmark::remove_challenge_bookmark(&self.ctx, persona_id, challenge_id)
            .await
            .map_err(GatewayError::into_rpc_err)?;
        Ok("success".to_string())
    }

    async fn get_player_ugc_limits(&self, extensions: &Extensions) -> RpcResult<PlayerUgcLimits> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::player::get_player_ugc_limits(&self.ctx, persona_id)
            .await
            .map_err(GatewayError::into_rpc_err)
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
            .map_err(GatewayError::into_rpc_err)
    }

    async fn create_time_trial(
        &self,
        extensions: &Extensions,
        data: String,
        meta: CreateTimeTrialMeta,
    ) -> RpcResult<UgcMeta> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::ugc::create_time_trial(&self.ctx, persona_id, data, meta)
            .await
            .map_err(GatewayError::into_rpc_err)
    }

    async fn finish_reach_this(&self, extensions: &Extensions, ugc_id: UgcId) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::ugc::finish_reach_this(&self.ctx, persona_id, ugc_id.id)
            .await
            .map_err(GatewayError::into_rpc_err)?;
        Ok("success".to_string())
    }

    async fn get_overview_reach_this_leaderboard(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        radius: Option<i32>,
    ) -> RpcResult<OverviewLeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::leaderboard::get_overview_ugc_leaderboard(
            &self.ctx,
            persona_id,
            ugc_id.id,
            UgcEntryType::ReachThis,
            Order::Asc,
            radius.unwrap_or(3),
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_overview_time_trial_leaderboard(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        radius: Option<i32>,
    ) -> RpcResult<OverviewLeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::leaderboard::get_overview_ugc_leaderboard(
            &self.ctx,
            persona_id,
            ugc_id.id,
            UgcEntryType::TimeTrial,
            Order::Asc,
            radius.unwrap_or(3),
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_hackable_billboard_friends_leaderboard(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        offset: Option<i64>,
        count: i64,
    ) -> RpcResult<LeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::leaderboard::get_challenge_friends_leaderboard(
            &self.ctx,
            persona_id,
            challenge_id,
            ChallengeEntryType::HackableBillboard,
            Order::Desc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_overview_runners_route_leaderboard(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        radius: Option<i32>,
    ) -> RpcResult<OverviewLeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::leaderboard::get_overview_challenge_leaderboard(
            &self.ctx,
            persona_id,
            challenge_id,
            ChallengeEntryType::RunnersRoute,
            Order::Asc,
            radius.unwrap_or(3),
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_runners_route_friends_leaderboard(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::leaderboard::get_challenge_friends_leaderboard(
            &self.ctx,
            persona_id,
            challenge_id,
            ChallengeEntryType::RunnersRoute,
            Order::Asc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }
    async fn get_runners_route_leaderboard(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::leaderboard::get_challenge_leaderboard(
            &self.ctx,
            persona_id,
            challenge_id,
            ChallengeEntryType::RunnersRoute,
            Order::Asc,
            offset.unwrap_or(3),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn finish_runners_route(
        &self,
        extensions: &Extensions,
        challenge_id: String,
        main_stat: i32,
        extra_stats: serde_json::Value,
        run_id: i32,
    ) -> RpcResult<Division> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::challenge::finish_runners_route(
            &self.ctx,
            persona_id,
            challenge_id,
            main_stat as i64,
            extra_stats,
            run_id,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn start_time_trial(&self, _extensions: &Extensions, ugc_id: UgcId) -> RpcResult<String> {
        let db = self.ctx.db();
        let ugc_uuid = uuid::Uuid::parse_str(&ugc_id.id)
            .map_err(|_| GatewayError::invalid_params("invalid UGC UUID").into_rpc_err())?;

        let ugc = ugc::Entity::find_by_id(ugc_uuid)
            .one(db)
            .await
            .map_err(GatewayError::from)
            .map_err(GatewayError::into_rpc_err)?;

        if ugc.is_none() {
            return Err(
                GatewayError::game(logic::GameErrorCode::NotFound, "UGC not found").into_rpc_err(),
            );
        }

        Ok("success".to_string())
    }

    async fn cancel_time_trial(
        &self,
        _extensions: &Extensions,
        _ugc_id: UgcId,
    ) -> RpcResult<String> {
        Ok("success".to_string())
    }

    async fn finish_time_trial(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        finish_time: i64,
        replay_upload_ticket: String,
        extra_stats: serde_json::Value,
        split_times: Vec<i64>,
    ) -> RpcResult<String> {
        let persona_id = *extensions.get::<i32>().unwrap();

        logic::ugc::finish_time_trial(
            &self.ctx,
            persona_id,
            ugc_id.id,
            finish_time,
            replay_upload_ticket,
            extra_stats,
            split_times,
        )
        .await
        .map_err(GatewayError::into_rpc_err)?;

        Ok("success".to_string())
    }

    async fn get_time_trial_leaderboard(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::leaderboard::get_ugc_leaderboard(
            &self.ctx,
            persona_id,
            ugc_id.id,
            UgcEntryType::TimeTrial,
            Order::Asc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_reach_this_leaderboard(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::leaderboard::get_ugc_leaderboard(
            &self.ctx,
            persona_id,
            ugc_id.id,
            UgcEntryType::ReachThis,
            Order::Asc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_time_trial_friends_leaderboard(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::leaderboard::get_ugc_friends_leaderboard(
            &self.ctx,
            persona_id,
            ugc_id.id,
            UgcEntryType::TimeTrial,
            Order::Asc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }

    async fn get_reach_this_friends_leaderboard(
        &self,
        extensions: &Extensions,
        ugc_id: UgcId,
        count: i64,
        offset: Option<i64>,
    ) -> RpcResult<LeaderboardResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::leaderboard::get_ugc_friends_leaderboard(
            &self.ctx,
            persona_id,
            ugc_id.id,
            UgcEntryType::ReachThis,
            Order::Asc,
            offset.unwrap_or(0),
            count,
        )
        .await
        .map_err(GatewayError::into_rpc_err)
    }
}
