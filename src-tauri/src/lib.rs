use tauri::Manager;

pub mod tunnel;

use crate::tunnel::manager::{
    get_config, update_settings, save_tunnel, delete_tunnel,
    start_tunnel, stop_tunnel, get_active_tunnels, TunnelManager
};

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Load configuration from disk
            let config = tunnel::manager::load_config_from_disk(app.handle());
            app.manage(TunnelManager::new(config));
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
            open_url
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
