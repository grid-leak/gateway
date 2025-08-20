use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee_proc_macros::rpc;

use crate::types::Friend;

#[rpc(server, namespace = "PamplonaAuthenticated", namespace_separator = ".")]
pub trait PamplonaAuthenticated {
    #[method(name = "getFriends")]
    async fn get_friends(&self) -> RpcResult<Vec<Friend>>;
}

pub struct PamplonaAuthenticatedImpl;

#[async_trait]
impl PamplonaAuthenticatedServer for PamplonaAuthenticatedImpl {
    async fn get_friends(&self) -> RpcResult<Vec<Friend>> {
        let friend = Friend {
            persona_id: "1004562044380".into(),
            name: "Flark321".into(),
        };

        Ok(vec![friend])
    }
}
