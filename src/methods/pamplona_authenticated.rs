use crate::{
    context::GatewayContext,
    logic,
    methods::map_err,
    models::{
        challenge::{HackableBillboardLeader, RunnersRouteDataResponse},
        customization::GhostDataInput,
        game_data::{InitialGameDataResponse, Inventory, Item, Kit},
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
    async fn get_initial_game_data(&self) -> RpcResult<InitialGameDataResponse>;

    #[method(name = "getInventory", with_extensions)]
    async fn get_inventory(&self) -> RpcResult<Inventory>;

    #[method(name = "getRunnersRouteData", with_extensions)]
    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
    ) -> RpcResult<Vec<RunnersRouteDataResponse>>;

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

    #[method(name = "getPersonaStats", with_extensions)]
    async fn get_persona_stats(&self) -> RpcResult<UserStats>;
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
    ) -> RpcResult<InitialGameDataResponse> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::ugc::get_initial_game_data(&self.ctx, persona_id).await
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
    ) -> RpcResult<Vec<RunnersRouteDataResponse>> {
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
        logic::customization::set_player_ghost(&self.ctx, persona_id, ghost_data)
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
        logic::customization::set_player_tag(&self.ctx, persona_id, tag_data)
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

    async fn get_persona_stats(&self, extensions: &Extensions) -> RpcResult<UserStats> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::stats::get_persona_stats(&self.ctx, persona_id).await
    }
}
