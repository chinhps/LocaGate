use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tokio::sync::oneshot;

use crate::tunnel::config::{AppConfig, TunnelConfig};

pub struct TunnelManagerState {
    pub config: AppConfig,
    pub active_tunnels: HashMap<String, oneshot::Sender<()>>,
}

pub struct TunnelManager(pub Mutex<TunnelManagerState>);

impl TunnelManager {
    pub fn new(config: AppConfig) -> Self {
        Self(Mutex::new(TunnelManagerState {
            config,
            active_tunnels: HashMap::new(),
        }))
    }

    pub fn stop_all(&self) {
        if let Ok(mut state) = self.0.lock() {
            for (_, stop_tx) in state.active_tunnels.drain() {
                let _ = stop_tx.send(());
            }
        }
    }
}

pub fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;
    Ok(app_data_dir.join("config.json"))
}

pub fn save_config_to_disk(app_handle: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path(app_handle)?;
    println!("[Backend] Saving config to path: {:?}", config_path);
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| {
                let err = format!("Failed to create config directory: {}", e);
                println!("[Backend] Error: {}", err);
                err
            })?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| {
            let err = format!("Failed to serialize config: {}", e);
            println!("[Backend] Error: {}", err);
            err
        })?;
    std::fs::write(&config_path, content)
        .map_err(|e| {
            let err = format!("Failed to write config file: {}", e);
            println!("[Backend] Error: {}", err);
            err
        })?;
    println!("[Backend] Config successfully saved to disk.");
    Ok(())
}

pub fn load_config_from_disk(app_handle: &AppHandle) -> AppConfig {
    if let Ok(config_path) = get_config_path(app_handle) {
        println!("[Backend] Loading config from path: {:?}", config_path);
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    println!("[Backend] Config loaded successfully: {:?}", config);
                    return config;
                }
            }
        }
    }
    println!("[Backend] Config file does not exist, using default config.");
    AppConfig::default()
}

#[tauri::command]
pub fn get_config(
    state: tauri::State<'_, TunnelManager>,
) -> Result<AppConfig, String> {
    println!("[Backend] Command get_config invoked");
    let state_lock = state.0.lock().map_err(|e| e.to_string())?;
    Ok(state_lock.config.clone())
}

#[tauri::command]
pub fn update_settings(
    state: tauri::State<'_, TunnelManager>,
    app_handle: AppHandle,
    worker_url: String,
    auth_token: String,
) -> Result<(), String> {
    println!("[Backend] Command update_settings invoked with worker_url: {}, auth_token: {}", worker_url, auth_token);
    let mut state_lock = state.0.lock().map_err(|e| e.to_string())?;
    state_lock.config.worker_url = worker_url;
    state_lock.config.auth_token = auth_token;
    save_config_to_disk(&app_handle, &state_lock.config)?;
    Ok(())
}

#[tauri::command]
pub fn save_tunnel(
    state: tauri::State<'_, TunnelManager>,
    app_handle: AppHandle,
    tunnel: TunnelConfig,
) -> Result<(), String> {
    let mut state_lock = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(pos) = state_lock.config.tunnels.iter().position(|t| t.id == tunnel.id) {
        state_lock.config.tunnels[pos] = tunnel;
    } else {
        state_lock.config.tunnels.push(tunnel);
    }
    save_config_to_disk(&app_handle, &state_lock.config)?;
    Ok(())
}

#[tauri::command]
pub fn delete_tunnel(
    state: tauri::State<'_, TunnelManager>,
    app_handle: AppHandle,
    id: String,
) -> Result<(), String> {
    let mut state_lock = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(stop_tx) = state_lock.active_tunnels.remove(&id) {
        let _ = stop_tx.send(());
    }
    state_lock.config.tunnels.retain(|t| t.id != id);
    save_config_to_disk(&app_handle, &state_lock.config)?;
    Ok(())
}

#[tauri::command]
pub fn start_tunnel(
    state: tauri::State<'_, TunnelManager>,
    app_handle: AppHandle,
    id: String,
) -> Result<(), String> {
    let mut state_lock = state.0.lock().map_err(|e| e.to_string())?;
    if state_lock.active_tunnels.contains_key(&id) {
        return Err("Tunnel is already running".to_string());
    }

    let config = state_lock.config.tunnels.iter()
        .find(|t| t.id == id)
        .ok_or_else(|| "Tunnel config not found".to_string())?
        .clone();

    let worker_url = state_lock.config.worker_url.clone();
    let auth_token = state_lock.config.auth_token.clone();

    let stop_tx = crate::tunnel::client::start_tunnel_task(
        app_handle,
        config,
        worker_url,
        auth_token,
    );

    state_lock.active_tunnels.insert(id, stop_tx);
    Ok(())
}

#[tauri::command]
pub fn stop_tunnel(
    state: tauri::State<'_, TunnelManager>,
    id: String,
) -> Result<(), String> {
    let mut state_lock = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(stop_tx) = state_lock.active_tunnels.remove(&id) {
        let _ = stop_tx.send(());
        Ok(())
    } else {
        Err("Tunnel is not running".to_string())
    }
}

#[tauri::command]
pub fn get_active_tunnels(
    state: tauri::State<'_, TunnelManager>,
) -> Result<Vec<String>, String> {
    let state_lock = state.0.lock().map_err(|e| e.to_string())?;
    Ok(state_lock.active_tunnels.keys().cloned().collect())
}
