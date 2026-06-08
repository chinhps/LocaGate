use dioxus::prelude::*;
use crate::components::{Button, TextInput};
use wasm_bindgen::JsCast;


#[component]
pub fn Settings(
    mut worker_url: Signal<String>,
    mut auth_token: Signal<String>,
) -> Element {
    let mut url_sig = use_signal(|| worker_url.read().clone());
    let mut token_sig = use_signal(|| auth_token.read().clone());
    let mut is_saving = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error_msg = use_signal(|| "".to_string());

    let handle_save = move |_| {
        let url = url_sig.read().clone();
        let token = token_sig.read().clone();
        
        is_saving.set(true);
        saved.set(false);
        error_msg.set("".to_string());

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateSettingsArgs {
            worker_url: String,
            auth_token: String,
        }

        spawn(async move {
            match crate::app::call_tauri::<(), _>("update_settings", &UpdateSettingsArgs {
                worker_url: url.clone(),
                auth_token: token.clone(),
            }).await {
                Ok(_) => {
                    worker_url.set(url);
                    auth_token.set(token);
                    saved.set(true);
                    is_saving.set(false);
                    
                    let mut saved_clone = saved.clone();
                    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                        saved_clone.set(false);
                    }) as Box<dyn FnMut()>);
                    if let Some(window) = web_sys::window() {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            3000,
                        );
                        closure.forget();
                    }
                }
                Err(e) => {
                    error_msg.set(format!("Failed to save: {}", e));
                    is_saving.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            style: "max-width: 600px; display: flex; flex-direction: column; gap: var(--spacing-lg);",
            div {
                class: "view-header",
                style: "padding: 0; border-bottom: none; height: auto; margin-bottom: var(--spacing-sm);",
                h2 { class: "view-title", "Settings" }
            }

            div {
                class: "settings-card",
                style: "display: flex; flex-direction: column; gap: var(--spacing-md);",

                div {
                    class: "settings-card-title",
                    "Relay Settings"
                }

                div {
                    class: "form-group",
                    label { class: "form-label", "Cloudflare Worker URL" }
                    TextInput {
                        value: "{url_sig}",
                        placeholder: "https://locagate-relay.yourname.workers.dev".to_string(),
                        oninput: move |e: FormEvent| url_sig.set(e.value())
                    }
                    span {
                        style: "font-size: 11px; color: var(--color-muted); margin-top: 4px; display: block;",
                        "The public base URL of your deployed Cloudflare Worker relay. Tunnels will map subdomains to this host."
                    }
                }

                div {
                    class: "form-group",
                    label { class: "form-label", "Authentication Token" }
                    TextInput {
                        value: "{token_sig}",
                        placeholder: "Secret token...".to_string(),
                        oninput: move |e: FormEvent| token_sig.set(e.value()),
                        r#type: "password".to_string()
                    }
                    span {
                        style: "font-size: 11px; color: var(--color-muted); margin-top: 4px; display: block;",
                        "A secret token configured in your wrangler.toml to authenticate tunnel connections."
                    }
                }

                div {
                    style: "display: flex; align-items: center; gap: var(--spacing-md); margin-top: var(--spacing-sm); flex-wrap: wrap;",
                    Button {
                        variant: "primary".to_string(),
                        onclick: handle_save,
                        disabled: *is_saving.read(),
                        "Save Settings"
                    }
                    if *is_saving.read() {
                        span {
                            style: "color: var(--color-muted); font-family: var(--font-mono); font-size: 13px;",
                            "SAVING..."
                        }
                    } else if *saved.read() {
                        span {
                            style: "color: var(--color-accent); font-family: var(--font-mono); font-size: 13px;",
                            "SETTINGS SAVED"
                        }
                    }
                }
                if !error_msg.read().is_empty() {
                    div {
                        style: "color: var(--color-error); font-size: 13px; font-family: var(--font-mono); background: var(--color-error-dim); border: 1px solid var(--color-error-border); padding: 8px 12px; border-radius: var(--radius-sm);",
                        "{error_msg}"
                    }
                }
            }

            // Guide and Deploy Card
            div {
                class: "settings-card",
                style: "display: flex; flex-direction: column; gap: var(--spacing-md);",
                div {
                    class: "settings-card-title",
                    "How to Setup Your Own Relay"
                }
                p {
                    style: "font-size: 13px; line-height: 1.5; color: var(--color-muted); margin: 0;",
                    "To host your own secure tunnel proxy, follow these steps to deploy and configure a Cloudflare Worker:"
                }
                div {
                    style: "display: flex; flex-direction: column; gap: 12px; font-size: 13px; line-height: 1.4;",
                    div {
                        strong { "1. Create Cloudflare Account: " }
                        span { "You need a free Cloudflare account. If you don't have one, " }
                        a {
                            href: "#",
                            style: "color: var(--color-accent); text-decoration: underline;",
                            onclick: move |e| {
                                e.prevent_default();
                                #[derive(serde::Serialize)]
                                struct OpenUrlArgs { url: String }
                                spawn(async move {
                                    let _ = crate::app::call_tauri::<(), _>("open_url", &OpenUrlArgs { url: "https://dash.cloudflare.com/sign-up".to_string() }).await;
                                });
                            },
                            "sign up here"
                        }
                        span { "." }
                    }
                    div {
                        strong { "2. Deploy the Worker: " }
                        span { "Deploy the relay server code to your Cloudflare account. You can deploy it with one click: " }
                        br {}
                        a {
                            href: "#",
                            style: "display: inline-block; margin-top: 6px;",
                            onclick: move |e| {
                                e.prevent_default();
                                #[derive(serde::Serialize)]
                                struct OpenUrlArgs { url: String }
                                spawn(async move {
                                    let _ = crate::app::call_tauri::<(), _>("open_url", &OpenUrlArgs { url: "https://deploy.workers.cloudflare.com/?url=https://github.com/chinhps/LocaGate".to_string() }).await;
                                });
                            },
                            img {
                                src: "https://deploy.workers.cloudflare.com/button.svg",
                                alt: "Deploy to Cloudflare Workers"
                            }
                        }
                    }
                    div {
                        strong { "3. Configure AUTH_TOKEN: " }
                        span { "Once deployed, go to your Cloudflare Workers dashboard, select your worker, go to " }
                        strong { "Settings > Variables" }
                        span { ", and add a variable named " }
                        code { style: "background: var(--color-bg-code); padding: 2px 4px; border-radius: 3px;", "AUTH_TOKEN" }
                        span { " with a secure password. Paste that same password into the 'Authentication Token' field above." }
                    }
                }
                div {
                    style: "display: flex; gap: var(--spacing-md); margin-top: var(--spacing-sm); flex-wrap: wrap;",
                    Button {
                        variant: "secondary".to_string(),
                        onclick: move |_| {
                            #[derive(serde::Serialize)]
                            struct OpenUrlArgs { url: String }
                            spawn(async move {
                                let _ = crate::app::call_tauri::<(), _>("open_url", &OpenUrlArgs { url: "https://dash.cloudflare.com/?to=/:account/workers-and-pages".to_string() }).await;
                            });
                        },
                        "Go to Workers Dashboard"
                    }
                    Button {
                        variant: "secondary".to_string(),
                        onclick: move |_| {
                            #[derive(serde::Serialize)]
                            struct OpenUrlArgs { url: String }
                            spawn(async move {
                                let _ = crate::app::call_tauri::<(), _>("open_url", &OpenUrlArgs { url: "https://github.com/chinhps/LocaGate".to_string() }).await;
                            });
                        },
                        "View Source Code"
                    }
                }
            }

            div {
                class: "settings-card",
                style: "display: flex; flex-direction: column; gap: var(--spacing-sm); font-family: var(--font-mono); font-size: 12px; color: var(--color-muted);",
                div {
                    class: "settings-card-title",
                    style: "font-family: var(--font-sans);",
                    "System Info"
                }
                div { "LocaGate Client: v0.1.2" }
                div { "Dioxus Core: v0.6.3" }
                div { "Tauri Framework: v2.11.0" }
                div {
                    " Powered by "
                    a {
                        href: "https://chinh.dev",
                        target: "_blank",
                        style: "color: var(--color-primary); text-decoration: underline;",
                        "ChinhDEV"
                    }
                }
            }
        }
    }
}
