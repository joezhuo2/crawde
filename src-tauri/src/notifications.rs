use crate::{activity::Session, preferences::Preferences, AppState};
use std::collections::BTreeMap;
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct Notifications {
    seen: BTreeMap<String, (String, u64)>,
}

pub fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    let preferences = state.preferences.lock().unwrap().clone();
    let sessions: Vec<Session> = {
        let activity = state.activity.lock().unwrap();
        activity
            .sessions
            .values()
            .filter(|s| !activity.is_dismissed(&s.id))
            .cloned()
            .collect()
    };
    let mut notifications = state.notifications.lock().unwrap();
    let now = crate::bridge::now();
    let quiet = crate::focus::quiet(&preferences);
    notifications
        .seen
        .retain(|id, _| sessions.iter().any(|s| &s.id == id));
    for session in sessions {
        let previous = notifications.seen.get(&session.id);
        let changed = previous.is_none_or(|(status, _)| status != &session.status);
        let remind = session.status == "approval"
            && preferences.reminders
            && previous.is_some_and(|(_, at)| now.saturating_sub(*at) >= 45_000);
        if changed || remind {
            let kind = match session.status.as_str() {
                "approval" if preferences.approval_sound => Some("approval"),
                "done" if preferences.completion_sound => Some("done"),
                "error" if preferences.error_sound => Some("error"),
                _ => None,
            };
            if let Some(kind) = kind.filter(|_| !quiet) {
                play(kind, &preferences);
            }
            notifications.seen.insert(session.id, (session.status, now));
        }
    }
}

// Synthesized PCM avoids external audio assets and browser autoplay restrictions.
pub fn wave(kind: &str, preferences: &Preferences) -> Vec<u8> {
    let notes: &[f32] = match kind {
        "approval" => &[660.0, 880.0, 660.0],
        "error" => &[330.0, 247.0],
        _ => &[523.25, 659.25, 783.99],
    };
    let rate = 22_050u32;
    let note_len = (rate as f32 * 0.105) as usize;
    let count = note_len * notes.len();
    let mut bytes = Vec::with_capacity(44 + count * 2);
    bytes.extend(b"RIFF");
    bytes.extend((36 + count as u32 * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 2).to_le_bytes());
    bytes.extend(2u16.to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((count as u32 * 2).to_le_bytes());
    for &frequency in notes {
        for sample in 0..note_len {
            let t = sample as f32 / rate as f32;
            let phase = t * frequency * std::f32::consts::TAU;
            let envelope =
                (sample as f32 / 120.0).min(1.0) * (1.0 - sample as f32 / note_len as f32).powi(2);
            let value = if preferences.sound_style == "retro" {
                phase.sin().signum() * 0.22
            } else {
                phase.sin() * 0.45
            };
            let pcm =
                (value * envelope * preferences.volume.clamp(0.0, 1.0) * i16::MAX as f32) as i16;
            bytes.extend(pcm.to_le_bytes());
        }
    }
    bytes
}

pub fn sound_file<'a>(kind: &str, preferences: &'a Preferences) -> &'a str {
    match kind {
        "approval" => &preferences.approval_sound_file,
        "error" => &preferences.error_sound_file,
        _ => &preferences.completion_sound_file,
    }
}

#[cfg(windows)]
fn play_file(path: &str, volume: f32) -> bool {
    use windows_sys::Win32::Media::Multimedia::mciSendStringW;
    let send = |command: String| {
        let wide: Vec<u16> = command.encode_utf16().chain([0]).collect();
        unsafe { mciSendStringW(wide.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut()) == 0 }
    };
    let alias = format!("crawde{}", uuid::Uuid::new_v4().simple());
    if !send(format!("open \"{path}\" type mpegvideo alias {alias}")) {
        return false;
    }
    let level = (volume.clamp(0.0, 1.0) * 1000.0) as u32;
    send(format!("setaudio {alias} volume to {level}"));
    let played = send(format!("play {alias} wait"));
    send(format!("close {alias}"));
    played
}

#[cfg(target_os = "macos")]
fn play_file(path: &str, volume: f32) -> bool {
    std::process::Command::new("afplay")
        .args(["-v", &volume.clamp(0.0, 1.0).to_string(), path])
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(not(any(windows, target_os = "macos")))]
fn play_file(_: &str, _: f32) -> bool {
    false
}

fn play(kind: &str, preferences: &Preferences) {
    if !preferences.sound || preferences.volume <= 0.0 {
        return;
    }
    let bytes = wave(kind, preferences);
    let file = sound_file(kind, preferences).to_string();
    let volume = preferences.volume;
    std::thread::spawn(move || {
        if !file.is_empty() && play_file(&file, volume) {
            return;
        }
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::Media::Audio::{
                PlaySoundW, SND_MEMORY, SND_NODEFAULT, SND_SYNC,
            };
            // Synchronous playback keeps the in-memory WAV alive for the entire call.
            PlaySoundW(
                bytes.as_ptr().cast(),
                std::ptr::null_mut(),
                SND_MEMORY | SND_NODEFAULT | SND_SYNC,
            );
        }
        #[cfg(target_os = "macos")]
        {
            let path = std::env::temp_dir().join(format!("crawde-{}.wav", uuid::Uuid::new_v4()));
            if std::fs::write(&path, &bytes).is_ok() {
                let _ = std::process::Command::new("afplay").arg(&path).status();
                let _ = std::fs::remove_file(&path);
            }
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = bytes;
    });
}

#[tauri::command]
pub fn test_sound(preferences: Preferences, kind: Option<String>) {
    play(kind.as_deref().unwrap_or("done"), &preferences);
}

#[tauri::command]
pub async fn pick_sound(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .set_title("Choose a sound")
        .add_filter("Sounds", &["wav", "mp3"])
        .blocking_pick_file()
        .and_then(|path| path.into_path().ok())
        .map(|path| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pcm_wave_has_valid_lengths_and_mutable_volume() {
        let mut prefs = Preferences::default();
        let bytes = wave("done", &prefs);
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(
            u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize,
            bytes.len() - 44
        );
        prefs.volume = 0.0;
        assert!(wave("approval", &prefs)[44..].iter().all(|b| *b == 0));
    }
}
