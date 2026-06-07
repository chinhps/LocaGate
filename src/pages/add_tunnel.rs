use dioxus::prelude::*;
use crate::config::{TunnelConfig, TunnelType};
use crate::components::{Button, TextInput};

#[component]
pub fn AddTunnel(
    onsave: EventHandler<TunnelConfig>,
) -> Element {
    let mut tunnel_type = use_signal(|| TunnelType::Port);
    let mut local_port = use_signal(|| "8000".to_string());
    let mut vdomain_addr = use_signal(|| "127.0.0.1:80".to_string());
    let mut host_header = use_signal(|| "mysite.local".to_string());
    let mut custom_name = use_signal(|| "".to_string());
    let mut custom_id = use_signal(|| "".to_string());

    let mut error_msg = use_signal(|| "".to_string());

    let computed_name = match *tunnel_type.read() {
        TunnelType::Port => format!("port-{}", local_port.read()),
        TunnelType::VirtualDomain => format!("vdom-{}", host_header.read().replace('.', "-")),
    };

    let computed_id = computed_name.to_lowercase().chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let handle_submit = move |_| {
        let final_name = if custom_name.read().trim().is_empty() {
            computed_name.clone()
        } else {
            custom_name.read().clone()
        };

        let final_id = if custom_id.read().trim().is_empty() {
            computed_id.clone()
        } else {
            custom_id.read().to_lowercase().chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>()
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
                h2 { class: "view-title", "Add New Tunnel" }
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
                            "Port Proxy"
                        }
                        Button {
                            variant: if *tunnel_type.read() == TunnelType::VirtualDomain { "primary".to_string() } else { "secondary".to_string() },
                            onclick: move |_| {
                                tunnel_type.set(TunnelType::VirtualDomain);
                                error_msg.set("".to_string());
                            },
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
                        placeholder: "e.g. {computed_id}".to_string(),
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
                        "Create Tunnel"
                    }
                }
            }
        }
    }
}
