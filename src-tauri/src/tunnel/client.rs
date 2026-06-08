use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::tunnel::config::TunnelConfig;

const MAX_BODY_CACHE_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLogEntry {
    pub tunnel_id: String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub category: String,
}

pub struct RequestLogStore(pub Mutex<Vec<RequestLogEntry>>);

impl RequestLogStore {
    pub fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub fn upsert(&self, entry: RequestLogEntry, max_logs: usize) {
        if let Ok(mut logs) = self.0.lock() {
            if let Some(pos) = logs.iter().position(|e| e.request_id == entry.request_id) {
                logs[pos] = entry;
            } else {
                logs.push(entry);
                if logs.len() > max_logs {
                    let excess = logs.len() - max_logs;
                    logs.drain(0..excess);
                }
            }
        }
    }

    pub fn list(&self) -> Vec<RequestLogEntry> {
        self.0.lock().map(|logs| logs.clone()).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut logs) = self.0.lock() {
            logs.clear();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDetails {
    pub url: String,
    pub local_url: String,
    pub request_headers: HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_headers: HashMap<String, String>,
    pub response_body: Option<String>,
    pub timestamp: u64,
}

pub struct LogCache(pub Mutex<HashMap<String, RequestDetails>>);

impl LogCache {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

pub fn categorize_request(method: &str, path: &str, headers: &HashMap<String, String>) -> String {
    let mut sec_fetch_dest = String::new();
    let mut sec_fetch_mode = String::new();
    let mut x_requested_with = String::new();
    let mut accept = String::new();
    let mut content_type = String::new();

    for (k, v) in headers {
        match k.to_lowercase().as_str() {
            "sec-fetch-dest" => sec_fetch_dest = v.to_lowercase(),
            "sec-fetch-mode" => sec_fetch_mode = v.to_lowercase(),
            "x-requested-with" => x_requested_with = v.to_lowercase(),
            "accept" => accept = v.to_lowercase(),
            "content-type" => content_type = v.to_lowercase(),
            _ => {}
        }
    }

    if x_requested_with == "xmlhttprequest" {
        return "fetch".to_string();
    }

    // Browser-native classification (matches DevTools Network filters)
    if !sec_fetch_dest.is_empty() {
        match sec_fetch_dest.as_str() {
            "document" | "iframe" => return "document".to_string(),
            "script" | "style" => return "css_js".to_string(),
            "image" => return "image".to_string(),
            // fetch(), XHR, sendBeacon all use dest "empty"
            "empty" => return "fetch".to_string(),
            _ => {}
        }
    }

    if sec_fetch_mode == "navigate" {
        return "document".to_string();
    }

    categorize_request_fallback(method, path, &accept, &content_type)
}

fn categorize_request_fallback(method: &str, path: &str, accept: &str, content_type: &str) -> String {
    let method_upper = method.to_uppercase();
    if matches!(
        method_upper.as_str(),
        "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS"
    ) {
        return "fetch".to_string();
    }

    if accept.contains("text/html") || content_type.contains("text/html") {
        return "document".to_string();
    }
    if accept.contains("text/css") || content_type.contains("text/css") {
        return "css_js".to_string();
    }
    if accept.contains("application/javascript")
        || accept.contains("text/javascript")
        || content_type.contains("application/javascript")
        || content_type.contains("text/javascript")
    {
        return "css_js".to_string();
    }
    if accept.contains("image/") || content_type.contains("image/") {
        return "image".to_string();
    }
    if accept.contains("application/json")
        || accept.contains("text/json")
        || content_type.contains("application/json")
        || content_type.contains("text/json")
        || accept.contains("application/xml")
        || content_type.contains("application/xml")
    {
        return "fetch".to_string();
    }

    let path_lower = path.to_lowercase();
    if path_lower.ends_with(".js")
        || path_lower.ends_with(".mjs")
        || path_lower.ends_with(".ts")
        || path_lower.ends_with(".css")
    {
        return "css_js".to_string();
    }
    if path_lower.ends_with(".png")
        || path_lower.ends_with(".jpg")
        || path_lower.ends_with(".jpeg")
        || path_lower.ends_with(".gif")
        || path_lower.ends_with(".svg")
        || path_lower.ends_with(".webp")
        || path_lower.ends_with(".ico")
    {
        return "image".to_string();
    }
    if path_lower.ends_with(".html") || path_lower.ends_with(".htm") {
        return "document".to_string();
    }

    // Typical fetch/XHR when Sec-Fetch headers are absent (curl, older clients)
    if accept.contains("*/*") || accept.is_empty() {
        return "fetch".to_string();
    }

    "other".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn classifies_fetch_api_with_sec_fetch() {
        let h = headers(&[
            ("sec-fetch-dest", "empty"),
            ("sec-fetch-mode", "cors"),
            ("accept", "application/json"),
        ]);
        assert_eq!(categorize_request("GET", "/api/users", &h), "fetch");
    }

    #[test]
    fn classifies_xhr_without_sec_fetch() {
        let h = headers(&[("x-requested-with", "XMLHttpRequest")]);
        assert_eq!(categorize_request("GET", "/api/data", &h), "fetch");
    }

    #[test]
    fn classifies_post_without_sec_fetch() {
        let h = headers(&[("content-type", "application/json")]);
        assert_eq!(categorize_request("POST", "/api/save", &h), "fetch");
    }

    #[test]
    fn classifies_script_tag() {
        let h = headers(&[("sec-fetch-dest", "script")]);
        assert_eq!(categorize_request("GET", "/bundle.js", &h), "css_js");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelResponse {
    #[serde(rename = "type")]
    pub msg_type: String, // "response"
    pub id: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

fn emit_request_log(app_handle: &AppHandle, entry: RequestLogEntry, max_logs: u32) {
    let limit = max_logs.max(1) as usize;
    if let Some(store) = app_handle.try_state::<RequestLogStore>() {
        store.upsert(entry.clone(), limit);
    }
    let _ = app_handle.emit("tunnel:request", entry);
}

fn build_log_entry(
    tunnel_id: &str,
    req: &TunnelRequest,
    status: u16,
    duration_ms: u64,
    timestamp: u64,
) -> RequestLogEntry {
    RequestLogEntry {
        tunnel_id: tunnel_id.to_string(),
        request_id: req.id.clone(),
        method: req.method.clone(),
        path: req.path.clone(),
        status,
        duration_ms,
        timestamp,
        category: categorize_request(&req.method, &req.path, &req.headers),
    }
}
fn emit_status(app_handle: &AppHandle, tunnel_id: &str, status: &str) {
    #[derive(Clone, Serialize)]
    struct StatusEvent {
        tunnel_id: String,
        status: String,
    }
    let _ = app_handle.emit(
        "tunnel:status",
        StatusEvent {
            tunnel_id: tunnel_id.to_string(),
            status: status.to_string(),
        },
    );
}

pub fn start_tunnel_task(
    app_handle: AppHandle,
    config: TunnelConfig,
    worker_url: String,
    auth_token: String,
    max_logs: u32,
) -> oneshot::Sender<()> {
    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
    let tunnel_id = config.id.clone();
    let local_addr = config.local_addr.clone();
    let host_header = config.host_header.clone();

    tauri::async_runtime::spawn(async move {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();

        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(30);

        emit_status(&app_handle, &tunnel_id, "connecting");

        loop {
            // Build WS URL
            let mut ws_url = worker_url.clone();
            if ws_url.starts_with("https://") {
                ws_url = ws_url.replace("https://", "wss://");
            } else if ws_url.starts_with("http://") {
                ws_url = ws_url.replace("http://", "ws://");
            } else {
                ws_url = format!("wss://{}", ws_url);
            }

            if ws_url.ends_with('/') {
                ws_url.pop();
            }
            ws_url = format!("{}/connect?id={}&token={}", ws_url, tunnel_id, auth_token);

            let connect_result = connect_async(&ws_url).await;

            match connect_result {
                Ok((ws_stream, _)) => {
                    emit_status(&app_handle, &tunnel_id, "connected");
                    backoff = Duration::from_secs(1); // Reset backoff

                    let (mut ws_write, mut ws_read) = ws_stream.split();
                    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

                    let mut stopped = false;

                    loop {
                        tokio::select! {
                            // 1. Message from client handler task to send back to DO
                            Some(response_json) = rx.recv() => {
                                if let Err(e) = ws_write.send(Message::Text(response_json)).await {
                                    eprintln!("WS write error: {}", e);
                                    break;
                                }
                            }
                            // 2. Incoming request from DO
                            msg_opt = ws_read.next() => {
                                match msg_opt {
                                    Some(Ok(Message::Text(text))) => {
                                        if let Ok(req) = serde_json::from_str::<TunnelRequest>(&text) {
                                            let http_client = http_client.clone();
                                            let local_addr = local_addr.clone();
                                            let host_header = host_header.clone();
                                            let tx = tx.clone();
                                            let app_handle = app_handle.clone();
                                            let tunnel_id = tunnel_id.clone();

                                            tauri::async_runtime::spawn(async move {
                                                let start_time = std::time::Instant::now();
                                                let pending_ts = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs();

                                                emit_request_log(
                                                    &app_handle,
                                                    build_log_entry(
                                                        &tunnel_id,
                                                        &req,
                                                        0,
                                                        0,
                                                        pending_ts,
                                                    ),
                                                    max_logs,
                                                );

                                                let res = handle_local_request(
                                                    &http_client,
                                                    &local_addr,
                                                    host_header.as_ref().map(|s| s.as_str()),
                                                    req.clone()
                                                ).await;
                                                let duration_ms = start_time.elapsed().as_millis() as u64;

                                                let (status, resp) = match res {
                                                    Ok(resp) => {
                                                        let status = resp.status;
                                                        let serialized = serde_json::to_string(&resp).unwrap_or_default();
                                                        let _ = tx.send(serialized);
                                                        (status, resp)
                                                    }
                                                    Err(err) => {
                                                        let status = 502;
                                                        let mut headers = HashMap::new();
                                                        headers.insert("content-type".to_string(), "text/plain".to_string());
                                                        let resp = TunnelResponse {
                                                            msg_type: "response".to_string(),
                                                            id: req.id.clone(),
                                                            status,
                                                            headers,
                                                            body: Some(STANDARD.encode(format!("LocaGate client error: {}", err))),
                                                        };
                                                        let serialized = serde_json::to_string(&resp).unwrap_or_default();
                                                        let _ = tx.send(serialized);
                                                        (status, resp)
                                                    }
                                                };

                                                let now = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs();

                                                let mut public_host = String::new();
                                                for (k, v) in &req.headers {
                                                    if k.to_lowercase() == "host" {
                                                        public_host = v.clone();
                                                        break;
                                                    }
                                                }
                                                let public_url = if !public_host.is_empty() {
                                                    format!("https://{}{}", public_host, req.path)
                                                } else {
                                                    req.path.clone()
                                                };

                                                let local_url = if local_addr.starts_with("http://") || local_addr.starts_with("https://") {
                                                    format!("{}{}", local_addr, req.path)
                                                } else {
                                                    format!("http://{}{}", local_addr, req.path)
                                                };

                                                if let Some(cache) = app_handle.try_state::<LogCache>() {
                                                    if let Ok(mut map) = cache.0.lock() {
                                                        if map.len() >= max_logs.max(1) as usize {
                                                            let oldest_key = map.iter()
                                                                .min_by_key(|(_, d)| d.timestamp)
                                                                .map(|(k, _)| k.clone());
                                                            if let Some(k) = oldest_key {
                                                                map.remove(&k);
                                                            }
                                                        }
                                                        map.insert(req.id.clone(), RequestDetails {
                                                            url: public_url,
                                                            local_url,
                                                            request_headers: req.headers.clone(),
                                                            request_body: req.body.clone(),
                                                            response_headers: resp.headers.clone(),
                                                            response_body: resp.body.as_ref().and_then(|body| {
                                                                let decoded = STANDARD.decode(body).ok()?;
                                                                if decoded.len() as u64 <= MAX_BODY_CACHE_BYTES {
                                                                    Some(body.clone())
                                                                } else {
                                                                    None
                                                                }
                                                            }),
                                                            timestamp: now,
                                                        });
                                                    }
                                                }

                                                emit_request_log(
                                                    &app_handle,
                                                    build_log_entry(
                                                        &tunnel_id,
                                                        &req,
                                                        status,
                                                        duration_ms,
                                                        now,
                                                    ),
                                                    max_logs,
                                                );
                                            });
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) | None => {
                                        break;
                                    }
                                    _ => {} // Ignore other message types (pings/binary)
                                }
                            }
                            // 3. Stop signal
                            _ = &mut stop_rx => {
                                stopped = true;
                                break;
                            }
                        }
                    }

                    emit_status(&app_handle, &tunnel_id, "disconnected");

                    if stopped {
                        emit_status(&app_handle, &tunnel_id, "stopped");
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("WS connection failed: {}. Retrying in {:?}", err, backoff);
                    emit_status(&app_handle, &tunnel_id, "connecting_error");

                    tokio::select! {
                        _ = tokio::time::sleep(backoff) => {
                            backoff = std::cmp::min(backoff * 2, max_backoff);
                        }
                        _ = &mut stop_rx => {
                            emit_status(&app_handle, &tunnel_id, "stopped");
                            break;
                        }
                    }
                }
            }
        }
    });

    stop_tx
}

async fn handle_local_request(
    client: &reqwest::Client,
    local_addr: &str,
    host_header: Option<&str>,
    req: TunnelRequest,
) -> Result<TunnelResponse, String> {
    let path = req.path;
    let url = if local_addr.starts_with("http://") || local_addr.starts_with("https://") {
        format!("{}{}", local_addr, path)
    } else {
        format!("http://{}{}", local_addr, path)
    };

    let method = reqwest::Method::from_bytes(req.method.as_bytes())
        .map_err(|e| format!("Invalid HTTP method: {}", e))?;

    let mut builder = client.request(method, &url);

    for (key, val) in req.headers {
        let key_lower = key.to_lowercase();
        // Skip hop-by-hop headers
        if key_lower == "host"
            || key_lower == "connection"
            || key_lower == "upgrade"
            || key_lower == "sec-websocket-key"
        {
            continue;
        }
        builder = builder.header(key, val);
    }

    if let Some(host) = host_header {
        builder = builder.header("Host", host);
    }

    if let Some(body_base64) = req.body {
        let body_bytes = STANDARD.decode(body_base64)
            .map_err(|e| format!("Failed to decode request body: {}", e))?;
        builder = builder.body(body_bytes);
    }

    let resp = builder.send().await
        .map_err(|e| format!("Local request failed: {}", e))?;

    let status = resp.status().as_u16();
    let mut headers = HashMap::new();
    for (key, value) in resp.headers().iter() {
        if let Ok(val_str) = value.to_str() {
            headers.insert(key.as_str().to_string(), val_str.to_string());
        }
    }

    let request_id = req.id;
    let resp_bytes = resp.bytes().await
        .map_err(|e| format!("Failed to read local response body: {}", e))?;
    let body_encoded = Some(STANDARD.encode(&resp_bytes));

    Ok(TunnelResponse {
        msg_type: "response".to_string(),
        id: request_id,
        status,
        headers,
        body: body_encoded,
    })
}
