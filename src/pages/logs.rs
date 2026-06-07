use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestLog {
    pub tunnel_id: String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: u64,
}

#[component]
pub fn Logs(
    logs: ReadOnlySignal<Vec<RequestLog>>,
    onclear: EventHandler<MouseEvent>,
) -> Element {
    let logs_list = logs.read();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: calc(100vh - 120px); gap: var(--spacing-md);",
            div {
                class: "view-header",
                style: "padding: 0; border-bottom: none; height: auto; margin-bottom: var(--spacing-sm); display: flex; justify-content: space-between; align-items: center;",
                h2 { class: "view-title", "Request Logs" }
                button {
                    class: "ds-btn ds-btn-secondary",
                    style: "padding: 6px 12px; font-size: 12px;",
                    onclick: move |e| onclear.call(e),
                    "Clear Logs"
                }
            }

            div {
                class: "logs-container",
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                div {
                    class: "logs-header",
                    span { class: "logs-title", style: "width: 50px; text-align: left;", "Method" }
                    span { class: "logs-title", style: "flex: 1; text-align: left; margin-left: 12px;", "Path" }
                    span { class: "logs-title", style: "width: 50px; text-align: right;", "Status" }
                    span { class: "logs-title", style: "width: 80px; text-align: right;", "Duration" }
                }

                div {
                    class: "logs-body",
                    if logs_list.is_empty() {
                        div {
                            style: "color: var(--color-muted); text-align: center; margin-top: var(--spacing-lg);",
                            "No requests captured yet. Start a tunnel and hit the public URL."
                        }
                    } else {
                        // Display logs with newest at the top
                        for log in logs_list.iter().rev() {
                            {
                                let status_class = match log.status {
                                    200..=299 => "status-2xx",
                                    300..=399 => "status-3xx",
                                    _ => "status-5xx",
                                };
                                rsx! {
                                    div {
                                        class: "log-entry",
                                        key: "{log.request_id}",
                                        span { class: "log-method {log.method}", style: "width: 50px; text-align: left;", "{log.method}" }
                                        span { class: "log-path", style: "flex: 1; text-align: left; margin-left: 12px;", "{log.path}" }
                                        span { class: "log-status {status_class}", style: "width: 50px; text-align: right;", "{log.status}" }
                                        span { class: "log-duration", style: "width: 80px; text-align: right;", "{log.duration_ms}ms" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
