use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{config::Config, models::audio_session::AudioSession, services::window_service};

#[derive(Clone, Copy)]
pub enum AppEvent {
    VolumeChange,
    MixerVisibilityChange,
    ConfigChange,
    ThemeChange,
    WindowHidden,
}

impl AppEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppEvent::VolumeChange => "volume-change-event",
            AppEvent::MixerVisibilityChange => "mixer-visibility-change-event",
            AppEvent::ConfigChange => "config-change-event",
            AppEvent::ThemeChange => "theme-change-event",
            AppEvent::WindowHidden => "window-hidden-event",
        }
    }
}

fn emit<S: Serialize + Clone>(app_handle: &AppHandle, event: AppEvent, payload: S) {
    if let Err(e) = app_handle.emit(event.as_str(), payload) {
        log::error!("Failed to emit {}: {}", event.as_str(), e);
    }
}

/// Notifies the UI of a volume change and shows the overlay, unless the mixer is already open.
pub fn emit_volume_change_event(audio_session: &AudioSession, app_handle: &AppHandle) {
    emit(app_handle, AppEvent::VolumeChange, audio_session);

    let mixer_visible = app_handle
        .get_webview_window("mixer")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);

    if !mixer_visible {
        window_service::show_overlay(app_handle);
    }
}

pub fn emit_mixer_visibility_change_event(visible: bool, app_handle: &AppHandle) {
    emit(app_handle, AppEvent::MixerVisibilityChange, visible);
}

pub fn emit_config_change_event(config: &Config, app_handle: &AppHandle) {
    emit(app_handle, AppEvent::ConfigChange, config);
    emit(app_handle, AppEvent::ThemeChange, config.system.theme.clone());
}

pub fn emit_window_hidden_event(app_handle: &AppHandle) {
    emit(app_handle, AppEvent::WindowHidden, ());
}
