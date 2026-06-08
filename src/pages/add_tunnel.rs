use dioxus::prelude::*;
use crate::config::{TunnelConfig, TunnelType};
use crate::components::{Button, TextInput};

fn generate_random_suffix() -> String {
    let val = (js_sys::Math::random() * 65536.0) as u32;
    format!("{:04x}", val)
}

#[component]
pub fn AddTunnel(
    onsave: EventHandler<TunnelConfig>,
    editing_tunnel: Option<TunnelConfig>,
    oncancel: EventHandler<MouseEvent>,
) -> Element {
    let is_editing = editing_tunnel.is_some();

    let initial_type = editing_tunnel.as_ref().map(|t| t.tunnel_type).unwrap_or(TunnelType::Port);
    let initial_port = editing_tunnel.as_ref().map(|t| {
        if t.tunnel_type == TunnelType::Port {
            t.local_addr.strip_prefix("127.0.0.1:").unwrap_or(&t.local_addr).to_string()
        } else {
            "8000".to_string()
        }
    }).unwrap_or_else(|| "8000".to_string());

    let initial_vdomain = editing_tunnel.as_ref().map(|t| {
        if t.tunnel_type == TunnelType::VirtualDomain {
            t.local_addr.clone()
        } else {
            "127.0.0.1:80".to_string()
        }
    }).unwrap_or_else(|| "127.0.0.1:80".to_string());

    let initial_host = editing_tunnel.as_ref().and_then(|t| t.host_header.clone()).unwrap_or_else(|| "mysite.local".to_string());
    let initial_name = editing_tunnel.as_ref().map(|t| t.name.clone()).unwrap_or_default();
    let initial_id = editing_tunnel.as_ref().map(|t| t.id.clone()).unwrap_or_default();

    let mut tunnel_type = use_signal(|| initial_type);
    let mut local_port = use_signal(|| initial_port);
    let mut vdomain_addr = use_signal(|| initial_vdomain);
    let mut host_header = use_signal(|| initial_host);
    let mut custom_name = use_signal(|| initial_name);
    let mut custom_id = use_signal(|| initial_id);

    let mut error_msg = use_signal(|| "".to_string());

    let computed_name = match *tunnel_type.read() {
        TunnelType::Port => format!("port-{}", local_port.read()),
        TunnelType::VirtualDomain => format!("vdom-{}", host_header.read().replace('.', "-")),
    };

    let computed_id = computed_name.to_lowercase().chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let computed_id_for_submit = computed_id.clone();

    let handle_submit = move |_| {
        let final_name = if custom_name.read().trim().is_empty() {
            computed_name.clone()
        } else {
            custom_name.read().clone()
        };

        let final_id = if is_editing {
            custom_id.read().clone()
        } else {
            let base_id = if custom_id.read().trim().is_empty() {
                computed_id_for_submit.clone()
            } else {
                custom_id.read().to_lowercase().chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-')
                    .collect::<String>()
            };

            if base_id.is_empty() {
                error_msg.set("Tunnel ID cannot be empty".to_string());
                return;
            }

            if base_id.contains("-tunnel-") {
                base_id
            } else {
                format!("{}-tunnel-{}", base_id, generate_random_suffix())
            }
        };

        if final_id.is_empty() {
            error_msg.set("Tunnel ID cannot be empty".to_string());
            return;
        }

        let (addr, header) = match *tunnel_type.read() {
            TunnelType::Port => {
                let port = local_port.read().trim().to_string();
                if port.is_empty() {
                    error_msg.set("Port cannot be empty".to_string());
                    return;
                }
                if port.parse::<u16>().is_err() {
                    error_msg.set("Invalid port number".to_string());
                    return;
                }
                (format!("127.0.0.1:{}", port), None)
            }
            TunnelType::VirtualDomain => {
                let addr = vdomain_addr.read().trim().to_string();
                let host = host_header.read().trim().to_string();
                if addr.is_empty() || host.is_empty() {
                    error_msg.set("Address and Host Header cannot be empty".to_string());
                    return;
                }
                (addr, Some(host))
            }
        };

        onsave.call(TunnelConfig {
            id: final_id,
            name: final_name,
            local_addr: addr,
            host_header: header,
            tunnel_type: *tunnel_type.read(),
        });
    };

    rsx! {
        div {
            style: "max-width: 500px; display: flex; flex-direction: column; gap: var(--spacing-lg);",
            div {
                class: "view-header",
                style: "padding: 0; border-bottom: none; height: auto; margin-bottom: var(--spacing-sm);",
                h2 { class: "view-title", if is_editing { "Edit Tunnel" } else { "Add New Tunnel" } }
            }

            div {
                class: "settings-card",
                style: "display: flex; flex-direction: column; gap: var(--spacing-md);",

                div {
                    class: "form-group",
                    label { class: "form-label", "Tunnel Type" }
                    div {
                        style: "display: flex; gap: var(--spacing-sm);",
                        Button {
                            variant: if *tunnel_type.read() == TunnelType::Port { "primary".to_string() } else { "secondary".to_string() },
                            onclick: move |_| {
                                tunnel_type.set(TunnelType::Port);
                                error_msg.set("".to_string());
                            },
                            disabled: is_editing,
                            "Port Proxy"
                        }
                        Button {
                            variant: if *tunnel_type.read() == TunnelType::VirtualDomain { "primary".to_string() } else { "secondary".to_string() },
                            onclick: move |_| {
                                tunnel_type.set(TunnelType::VirtualDomain);
                                error_msg.set("".to_string());
                            },
                            disabled: is_editing,
                            "Virtual Domain"
                        }
                    }
                }

                if *tunnel_type.read() == TunnelType::Port {
                    div {
                        class: "form-group",
                        label { class: "form-label", "Local Port" }
                        TextInput {
                            value: "{local_port}",
                            placeholder: "e.g. 8000".to_string(),
                            oninput: move |e: FormEvent| local_port.set(e.value()),
                            r#type: "number".to_string()
                        }
                    }
                } else {
                    div {
                        style: "display: flex; gap: var(--spacing-sm);",
                        div {
                            class: "form-group",
                            style: "flex: 1;",
                            label { class: "form-label", "Local Address" }
                            TextInput {
                                value: "{vdomain_addr}",
                                placeholder: "e.g. 127.0.0.1:80".to_string(),
                                oninput: move |e: FormEvent| vdomain_addr.set(e.value())
                            }
                        }
                        div {
                            class: "form-group",
                            style: "flex: 1;",
                            label { class: "form-label", "Host Header" }
                            TextInput {
                                value: "{host_header}",
                                placeholder: "e.g. mysite.local".to_string(),
                                oninput: move |e: FormEvent| host_header.set(e.value())
                            }
                        }
                    }
                }

                div {
                    class: "form-group",
                    label { class: "form-label", "Tunnel Name (Optional)" }
                    TextInput {
                        value: "{custom_name}",
                        placeholder: "e.g. {computed_name}".to_string(),
                        oninput: move |e: FormEvent| custom_name.set(e.value())
                    }
                }

                div {
                    class: "form-group",
                    label { class: "form-label", "Tunnel ID / Subdomain (Optional)" }
                    TextInput {
                        value: "{custom_id}",
                        placeholder: if is_editing { "Tunnel ID cannot be changed" } else { "e.g. {computed_id}-tunnel-xxxx" },
                        disabled: is_editing,
                        oninput: move |e: FormEvent| custom_id.set(e.value())
                    }
                }

                if !error_msg.read().is_empty() {
                    div {
                        style: "color: var(--color-error); font-size: 13px; font-family: var(--font-mono); background: var(--color-error-dim); border: 1px solid var(--color-error-border); padding: 8px 12px; border-radius: var(--radius-sm);",
                        "{error_msg}"
                    }
                }

                div {
                    style: "margin-top: var(--spacing-sm); display: flex; gap: var(--spacing-sm);",
                    Button {
                        variant: "primary".to_string(),
                        onclick: handle_submit,
                        if is_editing { "Save Changes" } else { "Create Tunnel" }
                    }
                    if is_editing {
                        Button {
                            variant: "secondary".to_string(),
                            onclick: move |e| oncancel.call(e),
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}
