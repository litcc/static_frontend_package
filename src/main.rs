pub(crate) mod web_service;
pub(crate) mod log;

use anyhow::Result;
use crate::web_service::ServerConfig;

#[tokio::main]
async fn main() ->Result<()>{
    log::init()?;
    let web = web_service::WebService::new(ServerConfig{
        port:8080,
        host:"0.0.0.0".to_owned(),
        timeout: 0,
    }).await;

    web.await_web_task().await;
    Ok(())
}
