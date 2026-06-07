use dioxus::prelude::*;
use crate::config::{TunnelConfig, TunnelType};
use wasm_bindgen::JsCast;

#[component]
pub fn Button(
    onclick: EventHandler<MouseEvent>,
    #[props(default = "primary".to_string())] variant: String,
    #[props(default = false)] disabled: bool,
    children: Element,
) -> Element {
    let variant_class = match variant.as_str() {
        "primary" => "ds-btn-primary",
        "secondary" => "ds-btn-secondary",
        "ghost" => "ds-btn-ghost",
        "danger" => "ds-btn-danger",
        _ => "ds-btn-primary",
    };
    rsx! {
        button {
            class: "ds-btn {variant_class}",
            onclick: move |e| { if !disabled { onclick.call(e); } },
            disabled: disabled,
            {children}
        }
    }
}

#[component]
pub fn TextInput(
    value: String,
    #[props(default = "".to_string())] placeholder: String,
    oninput: EventHandler<FormEvent>,
    #[props(default = "text".to_string())] r#type: String,
) -> Element {
    let t = r#type.clone();
    rsx! {
        input {
            class: "ds-input-text",
            r#type: "{t}",
            value: "{value}",
            placeholder: "{placeholder}",
            oninput: move |e| oninput.call(e)
        }
    }
}

#[component]
pub fn StatusPill(status: String) -> Element {
    let status_class = status.to_lowercase();
    let display_status = match status.as_str() {
        "connecting_error" => "CONN ERR",
        "connecting" => "CONNECTING",
        "connected" => "ONLINE",
        "disconnected" => "OFFLINE",
        "stopped" => "STOPPED",
        _ => "IDLE",
    };

    rsx! {
        div {
            class: "ds-status-pill {status_class}",
            span { class: "ds-status-dot" }
            "{display_status}"
        }
    }
}

#[component]
pub fn MonoLabel(text: String, #[props(default = false)] copyable: bool) -> Element {
    let mut copied = use_signal(|| false);
    let text_to_copy = text.clone();

    let handle_copy = move |_| {
        if copyable {
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                let _ = clipboard.write_text(&text_to_copy);
                copied.set(true);
                
                let mut copied_sig = copied.clone();
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    copied_sig.set(false);
                }) as Box<dyn FnMut()>);
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    2000,
                );
                closure.forget();
            }
        }
    };

    rsx! {
        span {
            style: "font-family: var(--font-mono); font-size: 13px; display: inline-flex; align-items: center; gap: 8px;",
            span { "{text}" }
            if copyable {
                button {
                    class: "ds-btn ds-btn-ghost",
                    style: "padding: 2px 6px; font-size: 10px; height: 20px;",
                    onclick: handle_copy,
                    if *copied.read() { "COPIED" } else { "COPY" }
                }
            }
        }
    }
}

#[component]
pub fn TunnelCard(
    config: TunnelConfig,
    status: String,
    onstart: EventHandler<MouseEvent>,
    onstop: EventHandler<MouseEvent>,
    ondelete: EventHandler<MouseEvent>,
    worker_url: String,
) -> Element {
    let is_active = status == "connected" || status == "connecting" || status == "connecting_error";
    let card_active_class = if is_active { "active-tunnel" } else { "" };

    let type_label = match config.tunnel_type {
        TunnelType::Port => "Port Proxy",
        TunnelType::VirtualDomain => "V-Domain",
    };

    // Formulate public URL
    let public_url = if worker_url.contains("localhost") || worker_url.contains("127.0.0.1") {
        format!("{}/t/{}/", worker_url, config.id)
    } else {
        if let Some(stripped) = worker_url.strip_prefix("https://") {
            format!("https://{}.{}", config.id, stripped)
        } else if let Some(stripped) = worker_url.strip_prefix("http://") {
            format!("http://{}.{}", config.id, stripped)
        } else {
            format!("https://{}.{}", config.id, worker_url)
        }
    };

    rsx! {
        div {
            class: "tunnel-card {card_active_class}",
            div {
                class: "tunnel-card-header",
                div {
                    h3 { class: "tunnel-card-title", "{config.name}" }
                    span { class: "tunnel-card-type", "{type_label}" }
                }
                StatusPill { status: status.clone() }
            }

            div {
                class: "tunnel-card-details",
                div {
                    class: "tunnel-detail-row",
                    span { class: "tunnel-detail-label", "Local Address" }
                    span { class: "tunnel-detail-val", "{config.local_addr}" }
                }
                if let Some(host) = config.host_header.clone() {
                    div {
                          class: "tunnel-detail-row",
                          span { class: "tunnel-detail-label", "Host Header" }
                          span { class: "tunnel-detail-val", "{host}" }
                    }
                }
                div {
                    class: "tunnel-detail-row",
                    span { class: "tunnel-detail-label", "Public URL" }
                    span {
                        class: "tunnel-detail-val",
                        a { href: "{public_url}", target: "_blank", "{public_url}" }
                    }
                }
            }

            div {
                class: "tunnel-card-actions",
                if is_active {
                    Button {
                        variant: "secondary".to_string(),
                        onclick: move |e| onstop.call(e),
                        "Stop"
                    }
                } else {
                    Button {
                        variant: "primary".to_string(),
                        onclick: move |e| onstart.call(e),
                        "Start"
                    }
                }
                Button {
                    variant: "ghost".to_string(),
                    onclick: move |e| ondelete.call(e),
                    disabled: is_active,
                    "Delete"
                }
            }
        }
    }
}
