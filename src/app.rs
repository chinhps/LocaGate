#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::collections::HashMap;

use crate::config::{AppConfig, TunnelConfig};
use crate::pages::{dashboard::Dashboard, add_tunnel::AddTunnel, settings::Settings, logs::Logs};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], catch)]
    async fn listen(event: &str, handler: &js_sys::Function) -> Result<JsValue, JsValue>;
}

pub async fn call_tauri<R, T>(cmd: &str, args: &T) -> Result<R, String>
where
    T: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let js_args = serde_wasm_bindgen::to_value(args)
        .map_err(|e| format!("Failed to serialize arguments: {:?}", e))?;

    let js_res = invoke(cmd, js_args).await
        .map_err(|e| {
            e.as_string().unwrap_or_else(|| "Unknown error in Tauri command".to_string())
        })?;

    let res = serde_wasm_bindgen::from_value::<R>(js_res)
        .map_err(|e| format!("Failed to deserialize response: {:?}", e))?;

    Ok(res)
}

#[derive(Clone, Serialize, Deserialize)]
struct StatusEvent {
    tunnel_id: String,
    status: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct RequestLogEvent {
    tunnel_id: String,
    request_id: String,
    method: String,
    path: String,
    status: u16,
    duration_ms: u64,
    timestamp: u64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ActivePage {
    Dashboard,
    AddTunnel,
    Logs,
    Settings,
}

pub fn App() -> Element {
    let mut current_page = use_signal(|| ActivePage::Dashboard);
    
    // Shared configurations
    let mut worker_url = use_signal(|| "".to_string());
    let mut auth_token = use_signal(|| "".to_string());
    let mut tunnels = use_signal(|| Vec::<TunnelConfig>::new());
    
    // Runtime statuses and request logs
    let mut statuses = use_signal(|| HashMap::<String, String>::new());
    let mut logs = use_signal(|| Vec::<crate::pages::logs::RequestLog>::new());

    // Load configurations and active statuses on start
    let _load_config = use_resource(move || async move {
        if let Ok(conf) = call_tauri::<AppConfig, _>("get_config", &()).await {
            worker_url.set(conf.worker_url);
            auth_token.set(conf.auth_token);
            tunnels.set(conf.tunnels);
        }

        if let Ok(active) = call_tauri::<Vec<String>, _>("get_active_tunnels", &()).await {
            let mut statuses_lock = statuses.write();
            for id in active {
                statuses_lock.insert(id, "connected".to_string());
            }
        }
    });

    // Event listener: Tunnel Status Changes
    use_effect(move || {
        let mut statuses_clone = statuses.clone();
        let closure = Closure::wrap(Box::new(move |event_obj: JsValue| {
            if let Ok(payload_val) = js_sys::Reflect::get(&event_obj, &JsValue::from_str("payload")) {
                if let Ok(event) = serde_wasm_bindgen::from_value::<StatusEvent>(payload_val) {
                    statuses_clone.write().insert(event.tunnel_id, event.status);
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        spawn(async move {
            let _ = listen("tunnel:status", closure.as_ref().unchecked_ref()).await;
            closure.forget();
        });
    });

    // Event listener: Relayed Request Logs
    use_effect(move || {
        let mut logs_clone = logs.clone();
        let closure = Closure::wrap(Box::new(move |event_obj: JsValue| {
            if let Ok(payload_val) = js_sys::Reflect::get(&event_obj, &JsValue::from_str("payload")) {
                if let Ok(event) = serde_wasm_bindgen::from_value::<RequestLogEvent>(payload_val) {
                    logs_clone.write().push(crate::pages::logs::RequestLog {
                        tunnel_id: event.tunnel_id,
                        request_id: event.request_id,
                        method: event.method,
                        path: event.path,
                        status: event.status,
                        duration_ms: event.duration_ms,
                        timestamp: event.timestamp,
                    });
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        spawn(async move {
            let _ = listen("tunnel:request", closure.as_ref().unchecked_ref()).await;
            closure.forget();
        });
    });

    #[derive(Serialize)]
    struct IdArgs {
        id: String,
    }

    #[derive(Serialize)]
    struct SaveTunnelArgs {
        tunnel: TunnelConfig,
    }

    // Control Handlers
    let handle_start = move |id: String| {
        spawn(async move {
            let _ = call_tauri::<(), _>("start_tunnel", &IdArgs { id }).await;
        });
    };

    let handle_stop = move |id: String| {
        spawn(async move {
            let _ = call_tauri::<(), _>("stop_tunnel", &IdArgs { id }).await;
        });
    };

    let handle_delete = move |id: String| {
        let mut tunnels_sig = tunnels.clone();
        spawn(async move {
            if call_tauri::<(), _>("delete_tunnel", &IdArgs { id: id.clone() }).await.is_ok() {
                tunnels_sig.write().retain(|t| t.id != id);
            }
        });
    };

    let handle_save_tunnel = move |new_tunnel: TunnelConfig| {
        let mut tunnels_sig = tunnels.clone();
        let mut page_sig = current_page.clone();
        spawn(async move {
            if call_tauri::<(), _>("save_tunnel", &SaveTunnelArgs { tunnel: new_tunnel.clone() }).await.is_ok() {
                let mut tunnels_lock = tunnels_sig.write();
                if let Some(pos) = tunnels_lock.iter().position(|t| t.id == new_tunnel.id) {
                    tunnels_lock[pos] = new_tunnel;
                } else {
                    tunnels_lock.push(new_tunnel);
                }
                page_sig.set(ActivePage::Dashboard);
            }
        });
    };



    let handle_clear_logs = move |_| {
        logs.write().clear();
    };

    let dashboard_class = if current_page() == ActivePage::Dashboard { "active" } else { "" };
    let add_tunnel_class = if current_page() == ActivePage::AddTunnel { "active" } else { "" };
    let logs_class = if current_page() == ActivePage::Logs { "active" } else { "" };
    let settings_class = if current_page() == ActivePage::Settings { "active" } else { "" };

    rsx! {
        link { rel: "stylesheet", href: "/assets/styles.css" }
        div {
            class: "app-container",
            
            // Sidebar Navigation
            aside {
                class: "sidebar",
                div {
                    div {
                        class: "brand-section",
                        h1 {
                            class: "brand-title",
                            span { class: "brand-dot" }
                            "LocaGate"
                        }
                    }
                    
                    ul {
                        class: "nav-list",
                        li {
                            class: "nav-item {dashboard_class}",
                            onclick: move |_| current_page.set(ActivePage::Dashboard),
                            "Dashboard"
                        }
                        li {
                            class: "nav-item {add_tunnel_class}",
                            onclick: move |_| current_page.set(ActivePage::AddTunnel),
                            "Add Tunnel"
                        }
                        li {
                            class: "nav-item {logs_class}",
                            onclick: move |_| current_page.set(ActivePage::Logs),
                            "Request Logs"
                        }
                        li {
                            class: "nav-item {settings_class}",
                            onclick: move |_| current_page.set(ActivePage::Settings),
                            "Settings"
                        }
                    }
                }
                
                div {
                    class: "sidebar-footer",
                    "status: online"
                }
            }
            
            // Main Content Area
            main {
                class: "main-content",
                div {
                    class: "view-body",
                    match current_page() {
                        ActivePage::Dashboard => rsx! {
                            Dashboard {
                                tunnels: tunnels,
                                statuses: statuses,
                                worker_url: worker_url,
                                onstart: handle_start,
                                onstop: handle_stop,
                                ondelete: handle_delete,
                            }
                        },
                        ActivePage::AddTunnel => rsx! {
                            AddTunnel {
                                onsave: handle_save_tunnel,
                            }
                        },
                        ActivePage::Logs => rsx! {
                            Logs {
                                logs: logs,
                                onclear: handle_clear_logs,
                            }
                        },
                        ActivePage::Settings => rsx! {
                            Settings {
                                worker_url: worker_url,
                                auth_token: auth_token,
                            }
                        }
                    }
                }
            }
        }
    }
}
