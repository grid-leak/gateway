use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;

use crate::models::{GetPlayerTagResponse, Tag, TagData};

#[rpc(server, namespace = "Pamplona", namespace_separator = ".")]
pub trait Pamplona {
    #[method(name = "getPlayerTag")]
    async fn get_player_tag(&self, persona_id: String) -> RpcResult<GetPlayerTagResponse>;
}

pub struct PamplonaImpl;

#[async_trait]
impl PamplonaServer for PamplonaImpl {
    async fn get_player_tag(&self, persona_id: String) -> RpcResult<GetPlayerTagResponse> {
        Ok(GetPlayerTagResponse {
            persona_id: persona_id.clone(),
            tag_data: TagData {
                frame: Tag {
                    tag: "2573550572".into(),
                },
                bg: Tag {
                    tag: "232356850".into(),
                },
                detail: Tag {
                    tag: "3420869487".into(),
                },
            },
        })
    }
}
