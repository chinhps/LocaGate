use std::collections::HashMap;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::tunnel::config::TunnelConfig;


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
                                                let res = handle_local_request(
                                                    &http_client,
                                                    &local_addr,
                                                    host_header.as_ref().map(|s| s.as_str()),
                                                    req.clone()
                                                ).await;
                                                let duration_ms = start_time.elapsed().as_millis() as u64;

                                                let (status, _) = match res {
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

                                                // Emit log event to Tauri frontend
                                                #[derive(Clone, Serialize)]
                                                struct RequestLogEvent {
                                                    tunnel_id: String,
                                                    request_id: String,
                                                    method: String,
                                                    path: String,
                                                    status: u16,
                                                    duration_ms: u64,
                                                    timestamp: u64,
                                                }

                                                let now = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs();

                                                let _ = app_handle.emit("tunnel:request", RequestLogEvent {
                                                    tunnel_id,
                                                    request_id: req.id,
                                                    method: req.method,
                                                    path: req.path,
                                                    status,
                                                    duration_ms,
                                                    timestamp: now,
                                                });
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

    let resp_bytes = resp.bytes().await
        .map_err(|e| format!("Failed to read local response body: {}", e))?;
    let body_encoded = STANDARD.encode(&resp_bytes);

    Ok(TunnelResponse {
        msg_type: "response".to_string(),
        id: req.id,
        status,
        headers,
        body: Some(body_encoded),
    })
}
