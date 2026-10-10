use tauri::Manager;

use services::window_service;
use tray::system_tray;
use utils::{key_bindings, logger, system_manager};

pub mod config;
mod tray {
    pub mod system_tray;
}
mod utils {
    pub mod key_bindings;
    pub mod logger;
    pub mod system_manager;
}
mod api {
    pub mod commands;
    pub mod events;
}
mod services {
    pub mod icon_service;
    pub mod volume_service;
    pub mod window_service;
}
mod models {
    pub mod audio_session;
}

pub fn run() {
    tauri::Builder::default()
        // Must be registered first. A second launch would install duplicate key hooks and tray icons.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("Maestro is already running, opening settings");
            window_service::open_settings(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle();

            logger::init(&app.path().app_log_dir()?);
            log::info!("Maestro v{}", env!("CARGO_PKG_VERSION"));

            config::init(&app.path().app_config_dir()?);

            system_manager::handle_debug_console(handle.clone());
            system_manager::handle_autostart(handle.clone());

            window_service::create_overlay(handle)?;
            window_service::create_mixer(handle)?;

            system_tray::initialize_tray(handle)?;

            key_bindings::initialize_key_listeners(handle.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::commands::get_all_sessions,
            api::commands::get_session,
            api::commands::set_session_volume,
            api::commands::toggle_session_mute,
            api::commands::log,
            api::commands::get_config,
            api::commands::set_config,
            window_service::get_taskbar_height
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
