use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestLog {
    pub tunnel_id: String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestDetails {
    pub url: String,
    pub local_url: String,
    pub request_headers: HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_headers: HashMap<String, String>,
    pub response_body: Option<String>,
    pub timestamp: u64,
}

fn pretty_print_json(raw: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Ok(pretty) = serde_json::to_string_pretty(&value) {
            return pretty;
        }
    }
    raw.to_string()
}

#[component]
pub fn Logs(
    logs: ReadOnlySignal<Vec<RequestLog>>,
    onclear: EventHandler<MouseEvent>,
) -> Element {
    let mut active_filter = use_signal(|| "all".to_string());
    let mut selected_request_id = use_signal(|| Option::<String>::None);
    let mut selected_details = use_signal(|| Option::<RequestDetails>::None);
    let mut is_loading_details = use_signal(|| false);
    let mut inspector_tab = use_signal(|| "headers".to_string());

    // Effect to fetch request details when selected_request_id changes
    use_effect(move || {
        let req_id = selected_request_id.read().clone();
        spawn(async move {
            if let Some(id) = req_id {
                is_loading_details.set(true);
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct GetDetailsArgs {
                    request_id: String,
                }
                match crate::app::call_tauri::<Option<RequestDetails>, _>("get_request_details", &GetDetailsArgs { request_id: id }).await {
                    Ok(details) => {
                        selected_details.set(details);
                    }
                    Err(err) => {
                        eprintln!("Failed to fetch request details: {}", err);
                        selected_details.set(None);
                    }
                }
                is_loading_details.set(false);
            } else {
                selected_details.set(None);
            }
        });
    });

    let logs_list = logs.read();
    let filter = active_filter.read().clone();
    let filtered_logs: Vec<RequestLog> = logs_list.iter()
        .filter(|log| {
            if filter == "all" {
                true
            } else {
                log.category == filter
            }
        })
        .cloned()
        .collect();

    // Evaluate matched log properties outside of rsx! block to prevent compiler syntax issues
    let req_id_opt = selected_request_id.read().clone();
    let mut matched_method = String::new();
    let mut matched_path = String::new();
    let mut matched_status = 0u16;
    let mut matched_duration = 0u64;
    let mut matched_status_class = "status-5xx";

    if let Some(ref req_id) = req_id_opt {
        if let Some(matched_log) = logs_list.iter().find(|l| &l.request_id == req_id) {
            matched_method = matched_log.method.clone();
            matched_path = matched_log.path.clone();
            matched_status = matched_log.status;
            matched_duration = matched_log.duration_ms;
            matched_status_class = match matched_status {
                200..=299 => "status-2xx",
                300..=399 => "status-3xx",
                _ => "status-5xx",
            };
        }
    }

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: calc(100vh - 120px); gap: var(--spacing-sm);",
            
            // Header Section
            div {
                class: "view-header",
                style: "padding: 0; border-bottom: none; height: auto; margin-bottom: 0; display: flex; justify-content: space-between; align-items: center;",
                h2 { class: "view-title", "Request Logs" }
                button {
                    class: "ds-btn ds-btn-secondary",
                    style: "padding: 6px 12px; font-size: 12px;",
                    onclick: move |e| {
                        selected_request_id.set(None);
                        onclear.call(e);
                    },
                    "Clear Logs"
                }
            }

            // Category filter tabs
            div {
                class: "log-filter-bar",
                button {
                    class: if active_filter.read().as_str() == "all" { "log-filter-tab active" } else { "log-filter-tab" },
                    onclick: move |_| active_filter.set("all".to_string()),
                    "All"
                }
                button {
                    class: if active_filter.read().as_str() == "fetch" { "log-filter-tab active" } else { "log-filter-tab" },
                    onclick: move |_| active_filter.set("fetch".to_string()),
                    "Fetch/XHR"
                }
                button {
                    class: if active_filter.read().as_str() == "document" { "log-filter-tab active" } else { "log-filter-tab" },
                    onclick: move |_| active_filter.set("document".to_string()),
                    "Document"
                }
                button {
                    class: if active_filter.read().as_str() == "css_js" { "log-filter-tab active" } else { "log-filter-tab" },
                    onclick: move |_| active_filter.set("css_js".to_string()),
                    "CSS/JS"
                }
                button {
                    class: if active_filter.read().as_str() == "image" { "log-filter-tab active" } else { "log-filter-tab" },
                    onclick: move |_| active_filter.set("image".to_string()),
                    "Images"
                }
                button {
                    class: if active_filter.read().as_str() == "other" { "log-filter-tab active" } else { "log-filter-tab" },
                    onclick: move |_| active_filter.set("other".to_string()),
                    "Other"
                }
            }

            // Split container for table and side inspector panel
            div {
                class: "logs-split-container",
                
                // Left Pane: Request logs table
                div {
                    class: "logs-left-pane",
                    div {
                        class: "logs-header",
                        span { class: "logs-title", style: "width: 60px; text-align: left;", "Method" }
                        span { class: "logs-title", style: "flex: 1; text-align: left; margin-left: 12px;", "Path" }
                        span { class: "logs-title", style: "width: 50px; text-align: right;", "Status" }
                        span { class: "logs-title", style: "width: 70px; text-align: right;", "Duration" }
                    }
                    div {
                        class: "logs-body",
                        if filtered_logs.is_empty() {
                            div {
                                style: "color: var(--color-muted); text-align: center; margin-top: var(--spacing-lg); font-family: var(--font-sans);",
                                "No requests captured for this filter."
                            }
                        } else {
                            // Show newest at the top
                            for log in filtered_logs.iter().rev() {
                                {
                                    let status_class = match log.status {
                                        200..=299 => "status-2xx",
                                        300..=399 => "status-3xx",
                                        _ => "status-5xx",
                                    };
                                    let is_active = selected_request_id.read().as_ref() == Some(&log.request_id);
                                    let active_class = if is_active { "log-entry active" } else { "log-entry" };
                                    
                                    // Extract owned values before rsx! block to prevent borrows
                                    let log_id = log.request_id.clone();
                                    let log_method = log.method.clone();
                                    let log_path = log.path.clone();
                                    let log_status = log.status;
                                    let log_duration = log.duration_ms;
                                    
                                    rsx! {
                                        div {
                                            class: "{active_class}",
                                            key: "{log_id}",
                                            onclick: move |_| {
                                                selected_request_id.set(Some(log_id.clone()));
                                                inspector_tab.set("headers".to_string());
                                            },
                                            span { class: "log-method {log_method}", "{log_method}" }
                                            span { class: "log-path", style: "margin-left: 12px;", "{log_path}" }
                                            span { class: "log-status {status_class}", "{log_status}" }
                                            span { class: "log-duration", "{log_duration}ms" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Right Pane: Inspector details (No nested rsx! macro call here)
                if req_id_opt.is_some() {
                    div {
                        class: "logs-right-inspector",
                        
                        // Inspector Header
                        div {
                            class: "inspector-header",
                            span { class: "inspector-title", "Inspect Request" }
                            button {
                                class: "inspector-close-btn",
                                onclick: move |_| selected_request_id.set(None),
                                "×"
                            }
                        }

                        // Tab bar
                        div {
                            class: "inspector-tabs",
                            button {
                                class: if inspector_tab.read().as_str() == "headers" { "inspector-tab active" } else { "inspector-tab" },
                                onclick: move |_| inspector_tab.set("headers".to_string()),
                                "Headers"
                            }
                            button {
                                class: if inspector_tab.read().as_str() == "request" { "inspector-tab active" } else { "inspector-tab" },
                                onclick: move |_| inspector_tab.set("request".to_string()),
                                "Request"
                            }
                            button {
                                class: if inspector_tab.read().as_str() == "response" { "inspector-tab active" } else { "inspector-tab" },
                                onclick: move |_| inspector_tab.set("response".to_string()),
                                "Response"
                            }
                        }

                        // Tab Body
                        div {
                            class: "inspector-body",
                            
                            if *is_loading_details.read() {
                                div {
                                    style: "display: flex; justify-content: center; align-items: center; height: 100px; color: var(--color-muted); font-family: var(--font-sans);",
                                    "Loading request details..."
                                }
                            } else if let Some(details) = selected_details.read().clone() {
                                if inspector_tab.read().as_str() == "headers" {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: var(--spacing-md); font-family: var(--font-mono); font-size: 12px;",
                                        
                                        // General Info
                                        div {
                                            div { style: "font-weight: bold; color: var(--color-primary); margin-bottom: 6px; font-size: 11px; text-transform: uppercase;", "General" }
                                                                            div {
                                                style: "display: grid; grid-template-columns: 130px 1fr; gap: 4px; background: rgba(0,0,0,0.15); padding: 8px; border: 1px solid var(--color-border); border-radius: var(--radius-sm);",
                                                span { style: "color: var(--color-muted);", "Request URL (Pub):" }
                                                span { style: "word-break: break-all; color: var(--color-ink); font-weight: 600;", "{details.url}" }
                                                span { style: "color: var(--color-muted);", "Target URL (Local):" }
                                                span { style: "word-break: break-all; color: var(--color-ink);", "{details.local_url}" }
                                                span { style: "color: var(--color-muted);", "Method:" }
                                                span { style: "font-weight: 600;", "{matched_method}" }
                                                span { style: "color: var(--color-muted);", "Path:" }
                                                span { style: "word-break: break-all; color: var(--color-ink);", "{matched_path}" }
                                                span { style: "color: var(--color-muted);", "Status:" }
                                                span { class: "{matched_status_class}", style: "font-weight: 600;", "{matched_status}" }
                                                span { style: "color: var(--color-muted);", "Duration:" }
                                                span { "{matched_duration}ms" }
                                            }
                                        }

                                        // Request Headers
                                        div {
                                            div { style: "font-weight: bold; color: var(--color-primary); margin-bottom: 6px; font-size: 11px; text-transform: uppercase;", "Request Headers" }
                                            if details.request_headers.is_empty() {
                                                div { style: "color: var(--color-muted); font-style: italic; padding-left: 8px;", "No headers present." }
                                            } else {
                                                div {
                                                    style: "display: flex; flex-direction: column; gap: 4px; background: rgba(0,0,0,0.15); padding: 8px; border: 1px solid var(--color-border); border-radius: var(--radius-sm); max-height: 200px; overflow-y: auto;",
                                                    for (k, v) in details.request_headers.iter() {
                                                        div {
                                                            style: "display: grid; grid-template-columns: 140px 1fr; gap: 8px; border-bottom: 1px solid rgba(255,255,255,0.02); padding-bottom: 2px;",
                                                            span { style: "color: var(--color-muted); font-weight: 600; word-break: break-all;", "{k}" }
                                                            span { style: "color: var(--color-ink); word-break: break-all;", "{v}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Response Headers
                                        div {
                                            div { style: "font-weight: bold; color: var(--color-primary); margin-bottom: 6px; font-size: 11px; text-transform: uppercase;", "Response Headers" }
                                            if details.response_headers.is_empty() {
                                                div { style: "color: var(--color-muted); font-style: italic; padding-left: 8px;", "No headers present." }
                                            } else {
                                                div {
                                                    style: "display: flex; flex-direction: column; gap: 4px; background: rgba(0,0,0,0.15); padding: 8px; border: 1px solid var(--color-border); border-radius: var(--radius-sm); max-height: 200px; overflow-y: auto;",
                                                    for (k, v) in details.response_headers.iter() {
                                                        div {
                                                            style: "display: grid; grid-template-columns: 140px 1fr; gap: 8px; border-bottom: 1px solid rgba(255,255,255,0.02); padding-bottom: 2px;",
                                                            span { style: "color: var(--color-muted); font-weight: 600; word-break: break-all;", "{k}" }
                                                            span { style: "color: var(--color-ink); word-break: break-all;", "{v}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if inspector_tab.read().as_str() == "request" {
                                    if let Some(body_base64) = &details.request_body {
                                        {
                                            let decoded = match base64::engine::general_purpose::STANDARD.decode(body_base64) {
                                                Ok(bytes) => {
                                                    String::from_utf8(bytes).unwrap_or_else(|_| "[Binary Payload / Non-UTF8 Text]".to_string())
                                                }
                                                Err(_) => "[Error: Failed to decode base64]".to_string()
                                            };
                                            let pretty = pretty_print_json(&decoded);
                                            rsx! {
                                                pre {
                                                    style: "margin: 0; padding: var(--spacing-sm); background: rgba(0, 0, 0, 0.2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); overflow-x: auto; white-space: pre-wrap; word-break: break-all; color: var(--color-ink); font-size: 11px; font-family: var(--font-mono); line-height: 1.4;",
                                                    "{pretty}"
                                                }
                                            }
                                        }
                                    } else {
                                        div { style: "color: var(--color-muted); font-style: italic; text-align: center; margin-top: var(--spacing-md); font-family: var(--font-sans);", "No request payload body" }
                                    }
                                } else if inspector_tab.read().as_str() == "response" {
                                    if let Some(body_base64) = &details.response_body {
                                        {
                                            let decoded = match base64::engine::general_purpose::STANDARD.decode(body_base64) {
                                                Ok(bytes) => {
                                                    String::from_utf8(bytes).unwrap_or_else(|_| "[Binary Payload / Non-UTF8 Text]".to_string())
                                                }
                                                Err(_) => "[Error: Failed to decode base64]".to_string()
                                            };
                                            let pretty = pretty_print_json(&decoded);
                                            rsx! {
                                                pre {
                                                    style: "margin: 0; padding: var(--spacing-sm); background: rgba(0, 0, 0, 0.2); border: 1px solid var(--color-border); border-radius: var(--radius-sm); overflow-x: auto; white-space: pre-wrap; word-break: break-all; color: var(--color-ink); font-size: 11px; font-family: var(--font-mono); line-height: 1.4;",
                                                    "{pretty}"
                                                }
                                            }
                                        }
                                    } else {
                                        div { style: "color: var(--color-muted); font-style: italic; text-align: center; margin-top: var(--spacing-md); font-family: var(--font-sans);", "No response payload body" }
                                    }
                                }
                            } else {
                                div {
                                    style: "color: var(--color-muted); text-align: center; margin-top: var(--spacing-md); font-family: var(--font-sans);",
                                    "Details not available."
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
