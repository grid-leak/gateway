use crate::{
    context::GatewayContext,
    logic,
    methods::map_err,
    models::{challenge::RunnersRouteDataResponse, game_data::InitialGameDataResponse},
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
};
use jsonrpsee_proc_macros::rpc;
use std::sync::Arc;

#[rpc(server, namespace = "PamplonaAuthenticated", namespace_separator = ".")]
pub trait PamplonaAuthenticated {
    #[method(name = "getInitialGameData", with_extensions)]
    async fn get_initial_game_data(&self) -> RpcResult<InitialGameDataResponse>;
    #[method(name = "getRunnersRouteData", with_extensions)]
    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
    ) -> RpcResult<Vec<RunnersRouteDataResponse>>;
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

    async fn get_runners_route_data(
        &self,
        extensions: &Extensions,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
    ) -> RpcResult<Vec<RunnersRouteDataResponse>> {
        let persona_id = *extensions.get::<i32>().unwrap();
        logic::challenge::get_runnersroute_data(&self.ctx, challenge_ids, data_types, persona_id)
            .await
            .map_err(map_err)
    }
}
