use crate::api::events::AppEvent;
use crate::config::{Config, get_config};
use tauri::{AppHandle, Listener};
use tauri_plugin_autostart::ManagerExt;
use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole, FreeConsole, GetConsoleWindow};

/// Keeps the "start with Windows" registration in sync with the config.
pub fn handle_autostart(app_handle: AppHandle) {
    // Dev builds would register the debug executable to launch at login.
    if cfg!(debug_assertions) {
        return;
    }

    fn set_autostart(app_handle: &AppHandle, enabled: bool) {
        let autostart_manager = app_handle.autolaunch();
        let result = if enabled {
            autostart_manager.enable()
        } else {
            autostart_manager.disable()
        };
        if let Err(e) = result {
            log::error!("Failed to set autostart to {}: {}", enabled, e);
        }
    }

    set_autostart(&app_handle, get_config().system.autostart);

    app_handle.clone().listen(AppEvent::ConfigChange.as_str(), move |event| {
        if let Ok(config) = serde_json::from_str::<Config>(event.payload()) {
            set_autostart(&app_handle, config.system.autostart);
        }
    });
}

/// Shows or hides the debug console according to the config.
pub fn handle_debug_console(app_handle: AppHandle) {
    fn set_debug_console(show_console: bool) {
        unsafe {
            let has_console = !GetConsoleWindow().is_invalid();

            if !show_console {
                if has_console {
                    let _ = FreeConsole();
                }
                return;
            }

            if has_console {
                log::info!("Console already attached");
            } else if AttachConsole(ATTACH_PARENT_PROCESS).is_ok() {
                log::info!("Console attached successfully");
            } else if let Err(e) = AllocConsole() {
                log::error!("Failed to allocate console: {}", e);
            }
        }
    }

    set_debug_console(get_config().system.show_console);

    app_handle.listen(AppEvent::ConfigChange.as_str(), move |event| {
        if let Ok(config) = serde_json::from_str::<Config>(event.payload()) {
            set_debug_console(config.system.show_console);
        }
    });
}
