use dioxus::prelude::*;
use crate::config::{TunnelConfig, TunnelType};
use wasm_bindgen::JsCast;
use qrcodegen::{QrCode, QrCodeEcc};

fn generate_qr_svg_path(text: &str) -> Option<(String, i32)> {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium).ok()?;
    let size = qr.size();
    let mut path = String::new();
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                path.push_str(&format!("M{} {}h1v1h-1z ", x, y));
            }
        }
    }
    Some((path, size))
}

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
    #[props(default = false)] disabled: bool,
) -> Element {
    let t = r#type.clone();
    rsx! {
        input {
            class: "ds-input-text",
            r#type: "{t}",
            value: "{value}",
            placeholder: "{placeholder}",
            disabled: disabled,
            oninput: move |e| oninput.call(e)
        }
    }
}

#[component]
pub fn StatusPill(status: String) -> Element {
    let status_class = match status.as_str() {
        "connected" => "online".to_string(),
        "connecting_error" => "error".to_string(),
        _ => status.to_lowercase(),
    };
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
    onedit: EventHandler<MouseEvent>,
    worker_url: String,
) -> Element {
    let mut show_qr = use_signal(|| false);
    let mut copied_qr_url = use_signal(|| false);

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
                    div {
                        style: "display: inline-flex; align-items: center; gap: 8px;",
                        span {
                            class: "tunnel-detail-val",
                            a { href: "{public_url}", target: "_blank", "{public_url}" }
                        }
                        if status == "connected" {
                            button {
                                class: "qr-btn",
                                title: "Show QR Code",
                                onclick: move |_| show_qr.set(true),
                                svg {
                                    view_box: "0 0 24 24",
                                    width: "14",
                                    height: "14",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    rect { x: "3", y: "3", width: "7", height: "7" }
                                    rect { x: "14", y: "3", width: "7", height: "7" }
                                    rect { x: "14", y: "14", width: "7", height: "7" }
                                    rect { x: "3", y: "14", width: "7", height: "7" }
                                }
                            }
                        }
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
                    onclick: move |e| onedit.call(e),
                    disabled: is_active,
                    "Edit"
                }
                Button {
                    variant: "ghost".to_string(),
                    onclick: move |e| ondelete.call(e),
                    disabled: is_active,
                    "Delete"
                }
            }
        }
        if show_qr() {
            {
                let qr_info = generate_qr_svg_path(&public_url);
                let (path_d, view_box, has_qr) = match qr_info {
                    Some((path, size)) => (path, format!("0 0 {} {}", size, size), true),
                    None => ("".to_string(), "0 0 100 100".to_string(), false)
                };
                let modal_url_copy = public_url.clone();
                let handle_modal_copy = move |_| {
                    if let Some(window) = web_sys::window() {
                        let clipboard = window.navigator().clipboard();
                        let _ = clipboard.write_text(&modal_url_copy);
                        copied_qr_url.set(true);
                        let mut copied_sig = copied_qr_url.clone();
                        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            copied_sig.set(false);
                        }) as Box<dyn FnMut()>);
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            2000,
                        );
                        closure.forget();
                    }
                };
                rsx! {
                    div {
                        class: "qr-modal-backdrop",
                        onclick: move |_| show_qr.set(false),
                        div {
                            class: "qr-modal-card",
                            onclick: move |e| e.stop_propagation(),
                            
                            div {
                                class: "qr-modal-header",
                                h3 { class: "qr-modal-title", "{config.name}" }
                                button {
                                    class: "qr-modal-close-x",
                                    onclick: move |_| show_qr.set(false),
                                    "×"
                                }
                            }
                            
                            div {
                                class: "qr-modal-body",
                                if has_qr {
                                    div {
                                        class: "qr-code-container",
                                        svg {
                                            view_box: "{view_box}",
                                            class: "qr-code-svg",
                                            path {
                                                d: "{path_d}",
                                                fill: "#000000"
                                            }
                                        }
                                    }
                                } else {
                                    p { "Failed to generate QR code" }
                                }
                                
                                div {
                                    class: "qr-modal-url-section",
                                    span { class: "qr-modal-url-label", "Public Address:" }
                                    div {
                                        class: "qr-modal-url-box",
                                        span { class: "qr-modal-url-text", "{public_url}" }
                                        button {
                                            class: "ds-btn ds-btn-ghost",
                                            style: "padding: 2px 6px; font-size: 10px; height: 20px; border: 1px solid var(--color-border);",
                                            onclick: handle_modal_copy,
                                            if copied_qr_url() { "COPIED" } else { "COPY" }
                                        }
                                    }
                                }
                            }
                            
                            div {
                                class: "qr-modal-footer",
                                Button {
                                    variant: "secondary".to_string(),
                                    onclick: move |_| show_qr.set(false),
                                    "Close"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
