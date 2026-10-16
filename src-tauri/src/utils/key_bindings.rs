use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Event, Listener};
use windows_key_listener::KeyListener;

use crate::api::events;
use crate::api::events::AppEvent;
use crate::config;
use crate::config::Config;
use crate::services::volume_service;
use crate::services::window_service;

const DEBOUNCE_INTERVAL: Duration = Duration::from_millis(25);
const VOLUME_STEP: i32 = 2;
const MASTER_SESSION: &str = "master";

#[derive(Debug, Clone, Copy)]
enum KeyAction {
    VolumeUp,
    VolumeDown,
    ToggleMute,
    ToggleMixer,
}

/// Registers global key bindings, and re-registers them whenever the config changes.
///
/// Key presses are handled on a dedicated worker thread. The low-level keyboard hook has to return quickly, or
/// Windows drops the event (and eventually removes the hook), so it only queues the action.
pub fn initialize_key_listeners(app_handle: AppHandle) {
    let (sender, receiver) = mpsc::channel::<KeyAction>();

    let worker_handle = app_handle.clone();
    thread::spawn(move || {
        for action in receiver {
            handle_action(action, &worker_handle);
        }
    });

    register_key_listeners(&KeyListener::new(), &sender, &config::get_config());

    app_handle.listen(AppEvent::ConfigChange.as_str(), {
        let app_handle = app_handle.clone();
        move |event: Event| {
            let Ok(config) = serde_json::from_str::<Config>(event.payload()) else {
                return;
            };

            // The hook must be (re)installed from the main thread, which runs the message loop it depends on.
            let sender = sender.clone();
            let result = app_handle.run_on_main_thread(move || {
                log::info!("Config changed, resetting key listeners");
                let key_listener = KeyListener::new();
                key_listener.unlisten();
                register_key_listeners(&key_listener, &sender, &config);
            });

            if let Err(e) = result {
                log::error!("Failed to reset key listeners: {}", e);
            }
        }
    });
}

fn register_key_listeners(key_listener: &KeyListener, sender: &Sender<KeyAction>, config: &Config) {
    let mut bindings = vec![
        ("VolumeUp".to_string(), KeyAction::VolumeUp),
        ("VolumeDown".to_string(), KeyAction::VolumeDown),
        ("VolumeMute".to_string(), KeyAction::ToggleMute),
    ];

    if config.mixer.enabled
        && let Some(hotkey) = config.mixer.hotkey.as_ref().filter(|hotkey| !hotkey.trim().is_empty())
    {
        bindings.push((hotkey.clone(), KeyAction::ToggleMixer));
    }

    for (key_chord, action) in bindings {
        log::info!("Registering {:?} hotkey: {}", action, key_chord);

        let sender = sender.clone();
        let callback = Arc::new(move || {
            let _ = sender.send(action);
            true
        });

        if let Err(e) = key_listener.listen(&key_chord, DEBOUNCE_INTERVAL, callback) {
            log::error!("Failed to register {:?} hotkey '{}': {}", action, key_chord, e);
        }
    }
}

fn handle_action(action: KeyAction, app_handle: &AppHandle) {
    let session = match action {
        KeyAction::VolumeUp => volume_service::adjust_session_volume(MASTER_SESSION, VOLUME_STEP),
        KeyAction::VolumeDown => volume_service::adjust_session_volume(MASTER_SESSION, -VOLUME_STEP),
        KeyAction::ToggleMute => volume_service::toggle_session_mute(MASTER_SESSION),
        KeyAction::ToggleMixer => {
            window_service::toggle_mixer(app_handle);
            return;
        }
    };

    if let Some(session) = session {
        events::emit_volume_change_event(&session, app_handle);
    }
}
