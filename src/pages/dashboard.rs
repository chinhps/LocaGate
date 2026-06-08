use dioxus::prelude::*;
use std::collections::HashMap;
use crate::config::TunnelConfig;
use crate::components::TunnelCard;

#[component]
pub fn Dashboard(
    tunnels: ReadOnlySignal<Vec<TunnelConfig>>,
    statuses: ReadOnlySignal<HashMap<String, String>>,
    worker_url: ReadOnlySignal<String>,
    onstart: EventHandler<String>,
    onstop: EventHandler<String>,
    ondelete: EventHandler<String>,
    onedit: EventHandler<TunnelConfig>,
) -> Element {
    let tunnels_list = tunnels.read();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: var(--spacing-lg);",
            div {
                class: "view-header",
                style: "padding: 0; border-bottom: none; height: auto; margin-bottom: var(--spacing-sm);",
                h2 { class: "view-title", "Dashboard" }
            }

            if tunnels_list.is_empty() {
                div {
                    class: "empty-state",
                    h3 { class: "empty-state-title", "No Tunnels Configured" }
                    p { class: "empty-state-desc", "Create your first tunnel to expose a local development port or virtual domain to the public internet." }
                }
            } else {
                div {
                    class: "tunnel-grid",
                    for config in tunnels_list.iter() {
                        {
                            let id = config.id.clone();
                            let status = statuses.read().get(&id).cloned().unwrap_or_else(|| "stopped".to_string());
                            let onstart_id = id.clone();
                            let onstop_id = id.clone();
                            let ondelete_id = id.clone();
                            let onedit_config = config.clone();
                            rsx! {
                                TunnelCard {
                                    key: "{config.id}",
                                    config: config.clone(),
                                    status: status,
                                    worker_url: worker_url.clone(),
                                    onstart: move |_| onstart.call(onstart_id.clone()),
                                    onstop: move |_| onstop.call(onstop_id.clone()),
                                    ondelete: move |_| ondelete.call(ondelete_id.clone()),
                                    onedit: move |_| onedit.call(onedit_config.clone()),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
