use jsonrpsee::{RpcModule, server::Server};
use std::{error::Error, net::SocketAddr};

mod methods;
mod types;

use crate::methods::{
    pamplona::{PamplonaImpl, PamplonaServer},
    pamplona_authenticated::{PamplonaAuthenticatedImpl, PamplonaAuthenticatedServer},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    let server = Server::builder().build(addr).await?;

    let mut methods = RpcModule::new(());

    methods.merge(PamplonaAuthenticatedImpl.into_rpc())?;
    methods.merge(PamplonaImpl.into_rpc())?;

    let handle = server.start(methods);

    handle.stopped().await;

    Ok(())
}
