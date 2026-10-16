use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tauri::image::Image;
use tauri::{
    AppHandle, Listener,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::api::events::AppEvent;
use crate::{services::window_service, utils};

const TRAY_ICON: &[u8] = include_bytes!("../../icons/speaker-32.png");

static WINDOW_LAST_HIDDEN: LazyLock<Mutex<Option<Instant>>> = LazyLock::new(|| Mutex::new(None));

pub fn initialize_tray(app_handle: &AppHandle) -> tauri::Result<()> {
    let version = app_handle.package_info().version.to_string();

    let version_item = MenuItem::with_id(app_handle, "version", format!("Version: {}", version), false, None::<&str>)?;
    let settings = MenuItem::with_id(app_handle, "settings", "Settings", true, None::<&str>)?;
    let open_logs = MenuItem::with_id(app_handle, "show_logs", "Logs", true, None::<&str>)?;
    let quit = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app_handle, &[&version_item, &settings, &open_logs, &quit])?;

    // Clicking the tray icon steals focus from the mixer, which hides it. Remember when that happened so the
    // same click doesn't immediately reopen it.
    app_handle.listen(AppEvent::WindowHidden.as_str(), |_| {
        *WINDOW_LAST_HIDDEN.lock().unwrap() = Some(Instant::now());
    });

    TrayIconBuilder::with_id("tray")
        .tooltip("Maestro")
        .icon(Image::from_bytes(TRAY_ICON)?)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show_logs" => {
                log::info!("Opening logs");
                utils::logger::open_log_dir();
            }
            "settings" => {
                log::info!("Opening settings");
                window_service::open_settings(app);
            }
            "quit" => {
                log::info!("Quitting Maestro");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                ..
            } = event
            else {
                return;
            };

            let grace_period = Duration::from_millis(300);
            let recently_hidden = WINDOW_LAST_HIDDEN
                .lock()
                .unwrap()
                .is_some_and(|last_hidden| last_hidden.elapsed() < grace_period);

            if recently_hidden {
                log::debug!("Ignoring tray click due to recent window hide");
                return;
            }

            window_service::toggle_mixer(tray.app_handle());
        })
        .build(app_handle)?;

    Ok(())
}
