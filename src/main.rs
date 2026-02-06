pub(crate) mod log;
pub(crate) mod web_service;

use crate::web_service::ServerConfig;
use anyhow::Result;

/// 当 MSVC 环境下使用 mimalloc
#[cfg(target_env = "msvc")]
pub use mimalloc::MiMalloc as Allocator;
/// 当非 MSVC 环境下使用 jemalloc
#[cfg(not(target_env = "msvc"))]
pub use tikv_jemallocator::Jemalloc as Allocator;

#[global_allocator]
static GLOBAL: Allocator = Allocator;

#[tokio::main]
async fn main() -> Result<()> {
    log::init()?;
    let port = std::env::var("PORT")
        .map(|i| {
            if i.is_empty() {
                8080
            } else {
                i.parse::<u16>().unwrap_or(8080)
            }
        })
        .unwrap_or(8080);

    let web = web_service::WebService::new(ServerConfig {
        port,
        host: "0.0.0.0".to_owned(),
        timeout: 0,
    })
    .await;

    web.await_web_task().await;
    Ok(())
}
