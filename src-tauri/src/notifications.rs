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
    let sessions: Vec<Session> = state
        .activity
        .lock()
        .unwrap()
        .sessions
        .values()
        .cloned()
        .collect();
    let mut notifications = state.notifications.lock().unwrap();
    let now = crate::bridge::now();
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
            if let Some(kind) = kind {
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

fn play(kind: &str, preferences: &Preferences) {
    if !preferences.sound || preferences.volume <= 0.0 {
        return;
    }
    let bytes = wave(kind, preferences);
    std::thread::spawn(move || {
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
        #[cfg(not(windows))]
        let _ = bytes;
    });
}

#[tauri::command]
pub fn test_sound(preferences: Preferences) {
    play("done", &preferences);
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
