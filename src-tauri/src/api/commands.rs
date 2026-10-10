use tauri::AppHandle;

use crate::{
    config::{self, Config},
    models::audio_session::AudioSession,
    services::volume_service,
    utils::logger,
};

use super::events;

#[tauri::command]
pub fn log(message: String, level: &str) {
    logger::log(message, level);
}

#[tauri::command]
pub fn get_all_sessions() -> Vec<AudioSession> {
    volume_service::get_all_sessions()
}

#[tauri::command]
pub fn get_session(session_name: &str) -> Option<AudioSession> {
    volume_service::get_session(session_name)
}

#[tauri::command]
pub fn set_session_volume(session_name: &str, volume: i32) -> Option<AudioSession> {
    volume_service::set_session_volume(session_name, volume)
}

#[tauri::command]
pub fn toggle_session_mute(app_handle: AppHandle, session_name: &str) -> Option<bool> {
    let session = volume_service::toggle_session_mute(session_name)?;
    events::emit_volume_change_event(&session, &app_handle);
    Some(session.mute)
}

#[tauri::command]
pub fn get_config() -> Config {
    config::get_config()
}

#[tauri::command]
pub fn set_config(config: Config, app_handle: AppHandle) -> Result<(), String> {
    config::set_config(config, &app_handle)
}
