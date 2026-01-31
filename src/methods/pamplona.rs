use std::sync::Arc;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;

use crate::{
    context::GatewayContext,
    models::social::{PlayerTagResponse, TagData, TagItem},
};

#[rpc(server, namespace = "Pamplona", namespace_separator = ".")]
pub trait Pamplona {
    #[method(name = "getPlayerTag")]
    async fn get_player_tag(&self, persona_id: String) -> RpcResult<PlayerTagResponse>;
    // #[method(name = "getPlayerInfo")]
    // async fn get_player_info(&self, persona_id: String) -> RpcResult<GetPlayerInfoResponse>;
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

    // async fn get_player_info(&self, persona_id: String) -> RpcResult<GetPlayerInfoResponse> {
    //     self.
    // }
}
