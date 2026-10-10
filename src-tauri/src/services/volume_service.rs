use std::collections::HashMap;
use windows_volume_control::{AudioController, CoinitMode, session::Session};

use crate::{config, models::audio_session::AudioSession};

fn get_audio_controller() -> AudioController {
    unsafe {
        let mut controller = AudioController::init(Some(CoinitMode::ApartmentThreaded));
        controller.load_current_sessions();
        controller
    }
}

/// Every session matching `session_name`. The `other` session matches everything not named in the config.
#[allow(clippy::borrowed_box)] // The Box comes from windows_volume_control
fn find_sessions<'a>(controller: &'a AudioController, session_name: &str) -> Vec<&'a Box<dyn Session>> {
    unsafe {
        if session_name.eq_ignore_ascii_case(config::OTHER_SESSION) {
            let defined_names = config::get_defined_session_names();
            let mut sessions = controller.get_all_sessions();
            sessions.retain(|session| !defined_names.contains(&session.get_name().to_lowercase()));
            sessions
        } else {
            controller.get_all_sessions_with_name(session_name.to_string())
        }
    }
}

fn to_scalar(volume: i32) -> f32 {
    volume.clamp(0, 100) as f32 / 100.0
}

pub fn get_all_sessions() -> Vec<AudioSession> {
    let controller = get_audio_controller();
    let mut session_map: HashMap<String, AudioSession> = HashMap::new();

    for session in unsafe { controller.get_all_sessions() } {
        let audio_session = AudioSession::from_session(session.as_ref());
        session_map.entry(audio_session.name.clone()).or_insert(audio_session);
    }

    session_map.into_values().collect()
}

pub fn get_session(session_name: &str) -> Option<AudioSession> {
    let controller = get_audio_controller();
    find_sessions(&controller, session_name)
        .into_iter()
        .next()
        .map(|session| AudioSession::from_session(session.as_ref()))
}

pub fn set_session_volume(session_name: &str, volume: i32) -> Option<AudioSession> {
    let controller = get_audio_controller();
    let sessions = find_sessions(&controller, session_name);

    if sessions.is_empty() {
        log::warn!("Set Volume: No Session Found: {}", session_name);
        return None;
    }

    for session in &sessions {
        log::info!("Setting {} volume -> {}", unsafe { session.get_name() }, volume.clamp(0, 100));
        unsafe { session.set_volume(to_scalar(volume)) };
    }

    sessions
        .into_iter()
        .next()
        .map(|session| AudioSession::from_session(session.as_ref()))
}

/// Changes the volume by `delta` percentage points, unmuting when the volume goes up.
pub fn adjust_session_volume(session_name: &str, delta: i32) -> Option<AudioSession> {
    let controller = get_audio_controller();
    let sessions = find_sessions(&controller, session_name);

    let Some(first) = sessions.first() else {
        log::warn!("Adjust Volume: No Session Found: {}", session_name);
        return None;
    };

    let current = unsafe { (first.get_volume() * 100.0).round() as i32 };
    let volume = (current + delta).clamp(0, 100);

    for session in &sessions {
        unsafe {
            session.set_volume(to_scalar(volume));
            if delta > 0 {
                session.set_mute(false);
            }
        }
    }

    log::info!("Adjusted {} volume {} -> {}", session_name, current, volume);
    sessions
        .into_iter()
        .next()
        .map(|session| AudioSession::from_session(session.as_ref()))
}

pub fn toggle_session_mute(session_name: &str) -> Option<AudioSession> {
    let controller = get_audio_controller();
    let sessions = find_sessions(&controller, session_name);

    let Some(first) = sessions.first() else {
        log::warn!("Toggle Mute: No Session Found: {}", session_name);
        return None;
    };

    let mute = !unsafe { first.get_mute() };
    for session in &sessions {
        unsafe { session.set_mute(mute) };
    }

    log::info!("Setting {} mute -> {}", session_name, mute);
    sessions
        .into_iter()
        .next()
        .map(|session| AudioSession::from_session(session.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_scalar_clamps_to_valid_range() {
        assert_eq!(to_scalar(-10), 0.0);
        assert_eq!(to_scalar(50), 0.5);
        assert_eq!(to_scalar(150), 1.0);
    }
}
