use crate::config::Config;
use crate::handlers::{get_ipv6, health, AppState};
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // 获取当前工作目录下的 config.json
    let config_path = std::env::current_dir()?.join("config.json");

    let config = Config::load_or_create(&config_path);
    tracing::info!("Server configured for port {}", config.port);

    let client = reqwest::Client::new();
    let state = Arc::new(AppState {
        config: config.clone(),
        client,
    });

    // 构建路由
    let app = Router::new()
        .route("/", get(get_ipv6))
        .route("/ipv6", get(get_ipv6))
        .route("/health", get(health))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
