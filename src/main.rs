pub(crate) mod web_service;
pub(crate) mod log;

use anyhow::Result;
use crate::web_service::ServerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    log::init()?;
    let port = std::env::var("PORT").map(|i| {
        if i.is_empty() {
            8080
        } else {
            i.parse::<u16>().unwrap_or(8080)
        }
    }).unwrap_or(8080);

    let web = web_service::WebService::new(ServerConfig {
        port,
        host: "0.0.0.0".to_owned(),
        timeout: 0,
    }).await;

    web.await_web_task().await;
    Ok(())
}
