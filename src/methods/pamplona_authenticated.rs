// use std::sync::Arc;

// use jsonrpsee::core::{RpcResult, async_trait};
// use jsonrpsee_proc_macros::rpc;

// use crate::{context::GatewayContext};

// #[rpc(server, namespace = "PamplonaAuthenticated", namespace_separator = ".")]
// pub trait PamplonaAuthenticated {
//     #[method(name = "getFriends")]
//     async fn get_friends(&self) -> RpcResult<Vec<Friend>>;
//     #[method(name = "getPlayerUGC")]
//     async fn get_player_ugc(&self) -> RpcResult<Vec<()>>;
// }

// pub struct PamplonaAuthenticatedImpl {
//     ctx: Arc<GatewayContext>,
// }

// impl PamplonaAuthenticatedImpl {
//     pub fn new(ctx: Arc<GatewayContext>) -> Self {
//         Self { ctx }
//     }
// }

// #[async_trait]
// impl PamplonaAuthenticatedServer for PamplonaAuthenticatedImpl {
//     async fn get_friends(&self) -> RpcResult<Vec<Friend>> {
//         let friend = Friend {
//             persona_id: "1004562044380".into(),
//             name: "Flark321".into(),
//         };

//         Ok(vec![friend])
//     }

//     async fn get_player_ugc(&self) -> RpcResult<Vec<()>> {
//         // TODO: grab db pool from context
//         // ugcs.filter(user_id.eq(1011786733))

//         Ok(vec![])
//     }
// }
