use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use windows::Win32::{
    Foundation::RECT,
    UI::WindowsAndMessaging::{GetSystemMetrics, SM_CYSCREEN, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW},
};

use crate::api::events;

pub fn create_overlay(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index-overlay.html".into()))
        .title("Overlay")
        .always_on_top(true)
        .skip_taskbar(true)
        .decorations(false)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()
}

pub fn create_mixer(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let window = WebviewWindowBuilder::new(app, "mixer", WebviewUrl::App("index-mixer.html".into()))
        .title("Mixer")
        .always_on_top(true)
        .skip_taskbar(true)
        .decorations(false)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()?;

    // Hide the mixer when it loses focus, ignoring the focus flicker that happens right as it is shown.
    let last_focus_time = Arc::new(Mutex::new(Instant::now()));
    let window_clone = window.clone();
    let app_handle = app.clone();
    window.on_window_event(move |event| {
        let tauri::WindowEvent::Focused(is_focused) = event else {
            return;
        };

        if *is_focused {
            *last_focus_time.lock().unwrap() = Instant::now();
            return;
        }

        if last_focus_time.lock().unwrap().elapsed() <= Duration::from_millis(100) {
            return;
        }

        let window = window_clone.clone();
        let app_handle = app_handle.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            if window.is_visible().unwrap_or(false) {
                log::debug!("Window focus lost -> closing");
                let _ = window.hide();
                events::emit_window_hidden_event(&app_handle);
            }
        });
    });

    Ok(window)
}

fn create_settings(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index-settings.html".into()))
        .title("Maestro | Settings")
        .decorations(true)
        .resizable(true)
        .focused(true)
        .always_on_top(false)
        .skip_taskbar(false)
        .visible(false)
        .build()
}

/// Opens the settings window, creating it if it was closed. The page shows itself once it has positioned itself.
pub fn open_settings(app: &AppHandle) {
    match app.get_webview_window("settings") {
        Some(window) => {
            let _ = window.show();
            let _ = window.set_focus();
        }
        None => {
            log::info!("Creating new settings window");
            if let Err(e) = create_settings(app) {
                log::error!("Failed to create settings window: {}", e);
            }
        }
    }
}

pub fn show_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.show();
    }
}

pub fn hide_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }
}

pub fn is_mixer_visible(app: &AppHandle) -> bool {
    app.get_webview_window("mixer")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

pub fn show_mixer(app: &AppHandle) {
    hide_overlay(app);
    events::emit_mixer_visibility_change_event(true, app);
    if let Some(window) = app.get_webview_window("mixer") {
        let _ = window.show();
    }
}

pub fn hide_mixer(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("mixer") {
        let _ = window.hide();
    }
    events::emit_mixer_visibility_change_event(false, app);
}

pub fn toggle_mixer(app: &AppHandle) {
    if is_mixer_visible(app) {
        hide_mixer(app);
    } else {
        show_mixer(app);
    }
}

#[tauri::command]
pub fn get_taskbar_height() -> i32 {
    let mut work_area = RECT::default();
    let result = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work_area as *mut RECT as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };

    if let Err(e) = result {
        log::warn!("Failed to get taskbar height: {}", e);
        return 0;
    }

    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    screen_height - (work_area.bottom - work_area.top)
}
