use windows_volume_control::session::Session;

use crate::services::icon_service;

#[derive(serde::Serialize, Clone)]
pub struct AudioSession {
    pub name: String,
    pub volume: i32,
    pub mute: bool,
    pub icon: Option<String>,
}

impl AudioSession {
    pub fn from_session(session: &dyn Session) -> Self {
        unsafe {
            AudioSession {
                name: session.get_name(),
                volume: (session.get_volume() * 100.0).round() as i32,
                mute: session.get_mute(),
                icon: icon_service::get_icon(session.get_pid()),
            }
        }
    }
}
