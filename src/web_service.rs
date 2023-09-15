use std::{
    net::{ToSocketAddrs},
    sync::{Arc, Mutex},
};
use std::time::Duration;

use anyhow::{Error, Result};
use axum::{Router};
use axum::body::{boxed, Full};
use axum::handler::HandlerWithoutStateExt;
use axum::http::{header, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;
use tokio::{sync::oneshot::Sender, task::JoinHandle};
use log::{info, warn};
use tower_http::compression::CompressionLayer;
use tower_http::compression::predicate::SizeAbove;
use tower_http::cors::{Any, CorsLayer};

pub struct WebService {
    /// 正常关闭 Web Service
    stop_handle: Arc<Mutex<Option<Sender<()>>>>,
    task: Mutex<Option<JoinHandle<()>>>,
}


#[derive(Debug, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl WebService {
    pub fn router() -> Router {
        let cors = CorsLayer::new()
            .allow_methods(Any)
            .allow_origin(Any)
            .allow_headers(Any)
            .max_age(Duration::from_secs(60 * 60));

        // 最小压缩文件大小
        let predicate = SizeAbove::new(256);

        let app = Router::new()
            .fallback_service(static_handler.into_service())
            .layer(cors)
            .layer(
                CompressionLayer::new()
                    .gzip(true)
                    .deflate(true)
                    .br(true)
                    .zstd(true)
                    .compress_when(predicate),
            );
        app
    }

    /// 启动新的 Web 服务
    pub fn new(config: &ServerConfig) -> Result<WebService> {
        let kk = (config.host.as_str(), config.port)
            .to_socket_addrs()?
            .next()
            .ok_or(Error::msg("错误"))?;
        info!("Listen to: {:?}", kk);

        let server = hyper::Server::bind(&kk).serve(WebService::router().into_make_service());

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let graceful = server.with_graceful_shutdown(async move {
            rx.await.ok();
        });

        let task = tokio::spawn(async move {
            // Await the `server` receiving the signal...
            if let Err(e) = graceful.await {
                warn!("server error: {}", e);
            } else {
                info!("Web Service stop!");
            }
        });

        Ok(WebService {
            stop_handle: Arc::new(Mutex::new(Some(tx))),
            task: Mutex::new(Some(task)),
        })
    }

    /// 等待 Web 服务任务
    pub async fn await_web_task(&self) {
        let _ = self.task.lock().unwrap().take().unwrap().await;
    }

    /// 关闭 Web 服务
    pub(crate) fn close(&self) {
        let binding = self.stop_handle.clone();
        let mut df = binding.lock().expect("获取锁失败");
        let _ = df.take().map(|tx| tx.send(()));
    }
}


#[derive(RustEmbed)]
#[folder = "dist"]
struct Asset;

pub struct StaticFile<T>(pub T);


impl<T> IntoResponse for StaticFile<T>
    where
        T: Into<String>,
{
    fn into_response(self) -> Response {
        let mut path = self.0.into();

        if path == "/" || path == "" {
            path = "index.html".to_string();
        }
        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder().header(header::CONTENT_TYPE, mime.as_ref()).body(body).unwrap()
            }
            None => {
                path = "index.html".to_string();
                let body = boxed(Full::from(Asset::get(path.as_str()).unwrap().data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder().header(header::CONTENT_TYPE, mime.as_ref()).body(body).unwrap()
            }
        }
    }
}


async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    StaticFile(path)
}