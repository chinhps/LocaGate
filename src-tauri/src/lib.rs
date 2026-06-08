use tauri::Manager;

pub mod tunnel;

use crate::tunnel::manager::{
    get_config, update_settings, save_tunnel, delete_tunnel,
    start_tunnel, stop_tunnel, get_active_tunnels, TunnelManager
};
use crate::tunnel::client::{LogCache, RequestDetails, RequestLogEntry, RequestLogStore};

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_request_details(
    cache: tauri::State<'_, LogCache>,
    request_id: String,
) -> Result<Option<RequestDetails>, String> {
    if let Ok(map) = cache.0.lock() {
        Ok(map.get(&request_id).cloned())
    } else {
        Err("Failed to lock log cache".to_string())
    }
}

#[tauri::command]
fn get_request_logs(
    store: tauri::State<'_, RequestLogStore>,
) -> Result<Vec<RequestLogEntry>, String> {
    Ok(store.list())
}

#[tauri::command]
fn clear_request_logs(store: tauri::State<'_, RequestLogStore>) -> Result<(), String> {
    store.clear();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Load configuration from disk
            let config = tunnel::manager::load_config_from_disk(app.handle());
            app.manage(TunnelManager::new(config));
            app.manage(LogCache::new());
            app.manage(RequestLogStore::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_settings,
            save_tunnel,
            delete_tunnel,
            start_tunnel,
            stop_tunnel,
            get_active_tunnels,
            open_url,
            get_request_details,
            get_request_logs,
            clear_request_logs
        ]);

    #[cfg(desktop)]
    {
        // Add desktop specific configurations if needed
    }

    let app = builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            // Stop all active tunnels on app exit
            if let Some(manager) = app_handle.try_state::<TunnelManager>() {
                manager.stop_all();
            }
        }
    });
}
