use std::{
    fmt::{Debug},
    sync::{
        Arc, Mutex,
    },
    time::Duration,
};

use anyhow::{ Result};
use axum::{ Router};

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::oneshot::{Receiver, Sender},
    task::JoinHandle,
};
use tokio::runtime::Handle;
use tower_http::compression::CompressionLayer;
use tower_http::compression::predicate::SizeAbove;
use tower_http::cors::{CorsLayer,Any as CorsAny};
use tower_http::services::{ServeDir, ServeFile};


#[derive(Clone)]
pub struct WebService {
    /// 正常关闭 Web Service
    stop_handle: Arc<Mutex<Option<Sender<()>>>>,
    task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Debug for WebService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebService").finish()
    }
}

/// WebService 配置
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
}


impl WebService {
    
    pub async fn new(config:ServerConfig) -> WebService {
        let w = WebService::listen(
            config.host.clone(),
            config.port.clone(),
            Self::router(&config),
        )
            .await
            .map_err(|e| {
                error!("{:?}", e);
                e
            })
            .expect("Api Web Server Error");
        info!("[WebService] 启动成功 {:?}",config);
        w
    }
    /// 路由
    pub fn router(#[allow(unused)] config: &ServerConfig) -> Router {


        // still don't compress gRPC
        // .and(NotForContentType::GRPC)
        // still don't compress images
        // .and(NotForContentType::IMAGES)
        // also don't compress JSON
        // .and(NotForContentType::const_new("application/json"));
        // .layer(Extension(pool));

        // 最小压缩文件大小
        let predicate = SizeAbove::new(256);

        let cors = CorsLayer::new()
            .allow_methods(CorsAny)
            .allow_origin(CorsAny)
            .allow_headers(CorsAny)
            .max_age(Duration::from_secs(60 * 60));

        let serve_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

        Router::new()
            // .nest_service("/", ServeDir::new("assets"))
            .fallback_service(serve_dir)
            .layer(
                CompressionLayer::new()
                    .gzip(true)
                    .deflate(true)
                    .br(true)
                    .zstd(true)
                    .compress_when(predicate),
            )
            .layer(cors)
        // .layer(HandleErrorLayer::new(error_handle::handle_error))
        // .layer(TimeoutLayer::new(Duration::from_secs(config.timeout)))
        // .layer(HandleErrorLayer::new(error_handle::handle))
        // .layer(TimeoutLayer::new(Duration::from_secs(config.timeout)))
    }

    // pub fn static_router(config: &ServerConfig, router: Router) -> Router {
    //     // still don't compress gRPC
    //     // .and(NotForContentType::GRPC)
    //     // still don't compress images
    //     // .and(NotForContentType::IMAGES)
    //     // also don't compress JSON
    //     // .and(NotForContentType::const_new("application/json"));
    //     // .layer(Extension(pool));
    // 
    //     // 最小压缩文件大小
    //     let predicate = SizeAbove::new(256);
    // 
    //     let cors = CorsLayer::new()
    //         .allow_methods(CorsAny)
    //         .allow_origin(CorsAny)
    //         .allow_headers(CorsAny)
    //         .max_age(Duration::from_secs(60 * 60));
    // 
    //     Router::new()
    //         .merge(router)
    //         .layer(
    //             ServiceBuilder::new()
    //                 // .layer(HandleErrorLayer::new(error_handle::handle))
    //                 .layer(
    //                     CompressionLayer::new()
    //                         .gzip(true)
    //                         .deflate(true)
    //                         .br(true)
    //                         .zstd(true)
    //                         .compress_when(predicate),
    //                 ),
    //         )
    //         .layer(cors)
    //     // .layer(TimeoutLayer::new(Duration::from_secs(config.timeout)))
    // }

    /// 启动新的 Web 服务
    pub(crate) async fn listen(
        host: String,
        port: u16,
        router: Router,
    ) -> Result<WebService> {
        let addr = format!("{}:{}", host.as_str(), port);

        let router = router;
        
        let handle = Handle::current();
        let (graceful_shutdown_handle, task) = std::thread::spawn(move || {
            // Using Handle::block_on to run async code in the new thread.
            handle.block_on(async {
                let listener = TcpListener::bind(&addr)
                    .await
                    .expect(format!("WebService {} Listener Error", &addr).as_str());

                let (tx, rx) = tokio::sync::oneshot::channel::<()>();
                // Run the server with graceful shutdown

                let back_loop = tokio::runtime::Handle::current().spawn(async move {
                    let _ = axum::serve(listener, router)
                        .with_graceful_shutdown(Self::shutdown_signal(rx))
                        .await;
                });

                (tx, back_loop)
            })
        })
            .join()
            .expect("Join error");

        Ok(WebService {
            stop_handle: Arc::new(Mutex::new(Some(graceful_shutdown_handle))),
            task: Arc::new(Mutex::new(Some(task))),
        })
    }

    async fn shutdown_signal(signal: Receiver<()>) {
        signal.await.ok();
    }

    /// 等待 Web 服务任务
    pub async fn await_web_task(&self) {
        let _ = self.task.lock().unwrap().take().unwrap().await;
    }

    /// 关闭 Web 服务
    #[allow(unused)]
    pub async fn close(&self) {
        let binding = self.stop_handle.clone();
        let mut df = binding.lock().expect("获取锁失败");
        let _ = df.take().map(|tx| tx.send(()));
        info!("Graceful stop WebService");
    }
}
