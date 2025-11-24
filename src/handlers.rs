use crate::config::Config;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use regex::Regex;
use serde_json::json;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

pub struct AppState {
    pub config: Config,
    pub client: reqwest::Client,
}

pub async fn get_ipv6(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let ipv6 = fetch_ipv6_from_urls(&state.config.urls, &state.client).await?;

    if let Some(ip) = ipv6 {
        tracing::info!("Found IPv6: {}", ip);
        Ok((StatusCode::OK, String::from(ip)))
    } else {
        tracing::warn!("Could not find IPv6 address from any URL");
        Err((
            StatusCode::NOT_FOUND,
            "Could not find IPv6 address".to_string(),
        ))
    }
}

async fn fetch_ipv6_from_urls(
    urls: &[String],
    client: &reqwest::Client,
) -> Result<Option<String>, (StatusCode, String)> {
    if urls.is_empty() {
        return Ok(None);
    }

    // 创建一个通道用于接收第一个成功的结果
    let (tx, mut rx) = mpsc::channel(urls.len());

    // 并发发起所有请求
    for url in urls {
        let url = url.clone();
        let client = client.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            match fetch_ipv6_from_url(&url, &client).await {
                Ok(Some(ip)) => {
                    if is_ipv6(&ip) {
                        tracing::info!("Got IPv6 from {}: {}", url, ip);
                        let _ = tx.send(Some(ip)).await;
                    }
                }
                Ok(None) => {
                    tracing::debug!("No IPv6 found from {}", url);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch from {}: {}", url, e);
                }
            }
        });
    }

    // 等待第一个成功的结果，最多等待15秒
    match timeout(Duration::from_secs(15), rx.recv()).await {
        Ok(Some(Some(ip))) => Ok(Some(ip)),
        Ok(Some(None)) | Ok(None) => Ok(None),
        Err(_) => {
            tracing::warn!("Timeout waiting for IPv6 response");
            Ok(None)
        }
    }
}

async fn fetch_ipv6_from_url(
    url: &str,
    client: &reqwest::Client,
) -> Result<Option<String>, String> {
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = response.text().await.map_err(|e| e.to_string())?;

    // 使用正则表达式直接从响应文本中提取IPv6地址
    // 匹配标准IPv6地址格式
    let ipv6_regex = Regex::new(
        r"(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})|:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]|(?:2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(?:25[0-5]|(?:2[0-4]|1{0,1}[0-9]){0,1}[0-9])|(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]|(?:2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(?:25[0-5]|(?:2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"
    ).map_err(|e| e.to_string())?;

    // 查找第一个有效的IPv6地址
    for capture in ipv6_regex.find_iter(&text) {
        let ip = capture.as_str();
        if is_ipv6(ip) {
            return Ok(Some(ip.to_string()));
        }
    }

    Ok(None)
}

fn is_ipv6(addr: &str) -> bool {
    addr.parse::<std::net::Ipv6Addr>().is_ok()
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}
