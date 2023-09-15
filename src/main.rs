mod web_service;


use anyhow::Result;
use crate::web_service::ServerConfig;

#[tokio::main]
async fn main() ->Result<()>{
    let web = web_service::WebService::new(&ServerConfig{
        port:8080,
        host:"0.0.0.0".to_owned()
    })?;

    web.await_web_task().await;
    Ok(())
}
