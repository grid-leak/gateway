use std::sync::Arc;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;

use crate::{
    context::GatewayContext,
    logic::challenge::get_runnersroute_data,
    methods::map_err,
    models::{
        challenge::RunnersRouteDataResponse,
        social::{PlayerTagResponse, TagData, TagItem},
    },
};

#[rpc(server, namespace = "Pamplona", namespace_separator = ".")]
pub trait Pamplona {
    #[method(name = "getPlayerTag")]
    async fn get_player_tag(&self, persona_id: String) -> RpcResult<PlayerTagResponse>;

    #[method(name = "getRunnersRouteData")]
    async fn get_runners_route_data(
        &self,
        challenge_ids: Vec<String>,
        data_types: Vec<String>,
        persona_id: String,
    ) -> RpcResult<Vec<RunnersRouteDataResponse>>;
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
    async fn get_player_tag(&self, persona_id: String) -> RpcResult<PlayerTagResponse> {
        Ok(PlayerTagResponse {
            persona_id: persona_id.clone(),
            tag_data: TagData {
                frame: TagItem {
                    tag: "2573550572".into(),
                },
                bg: TagItem {
                    tag: "232356850".into(),
                },
                detail: TagItem {
                    tag: "3420869487".into(),
                },
            },
        })
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

        get_runnersroute_data(&self.ctx, challenge_ids, data_types, pid)
            .await
            .map_err(map_err)
    }
}
