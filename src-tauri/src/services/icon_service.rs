use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Base64 PNG icons keyed by process id. Extracting an icon is slow, and sessions are re-read on every volume change.
static ICON_CACHE: LazyLock<Mutex<HashMap<u32, Option<String>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_icon(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }

    if let Some(icon) = ICON_CACHE.lock().unwrap().get(&pid) {
        return icon.clone();
    }

    let icon = windows_icons::get_icon_base64_by_process_id(pid)
        .inspect_err(|e| log::debug!("No icon for process {}: {}", pid, e))
        .ok();

    ICON_CACHE.lock().unwrap().insert(pid, icon.clone());
    icon
}
