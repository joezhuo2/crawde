use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub enabled: bool,
    pub startup: bool,
    pub hotkey: String,
    pub theme: String,
    pub scale: u32,
    pub roaming: bool,
    pub reduced_motion: bool,
    pub sound: bool,
    pub volume: f32,
    pub sound_style: String,
    pub approval_sound: bool,
    pub completion_sound: bool,
    pub error_sound: bool,
    pub reminders: bool,
    pub open_target: String,
    pub check_on_open: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            enabled: true,
            startup: true,
            hotkey: "Ctrl+Alt+P".into(),
            theme: "system".into(),
            scale: 3,
            roaming: true,
            reduced_motion: false,
            sound: true,
            volume: 0.45,
            sound_style: "soft".into(),
            approval_sound: true,
            completion_sound: true,
            error_sound: true,
            reminders: true,
            open_target: "desktop".into(),
            check_on_open: true,
        }
    }
}

pub fn load(app: &AppHandle) -> Preferences {
    app.path()
        .app_config_dir()
        .ok()
        .and_then(|d| std::fs::read(d.join("preferences.json")).ok())
        .and_then(|s| serde_json::from_slice(&s).ok())
        .unwrap_or_default()
}

pub fn persist(app: &AppHandle, preferences: &Preferences) -> Result<(), String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let staging = dir.join("preferences.tmp");
    std::fs::write(
        &staging,
        serde_json::to_vec_pretty(preferences).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(staging, dir.join("preferences.json")).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_preferences(app: AppHandle, preferences: Preferences) -> Result<(), String> {
    if !(2..=4).contains(&preferences.scale)
        || !preferences.volume.is_finite()
        || !(0.0..=1.0).contains(&preferences.volume)
        || !["system", "light", "dark"].contains(&preferences.theme.as_str())
        || !["soft", "retro"].contains(&preferences.sound_style.as_str())
        || !["desktop", "terminal"].contains(&preferences.open_target.as_str())
    {
        return Err("Invalid preference value".into());
    }
    preferences
        .hotkey
        .parse::<Shortcut>()
        .map_err(|e| format!("Invalid shortcut: {e}"))?;
    let state = app.state::<crate::AppState>();
    let previous = state.preferences.lock().map_err(|e| e.to_string())?.clone();
    if previous.hotkey != preferences.hotkey {
        app.global_shortcut()
            .on_shortcut(preferences.hotkey.as_str(), |app, _, event| {
                if event.state == ShortcutState::Pressed {
                    let _ = crate::lifecycle::toggle(app);
                }
            })
            .map_err(|e| format!("Shortcut is unavailable: {e}"))?;
    }
    let apply = || -> Result<(), String> {
        if preferences.startup {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        }
        .map_err(|e| e.to_string())?;
        persist(&app, &preferences)
    };
    if let Err(error) = apply() {
        if previous.hotkey != preferences.hotkey {
            let _ = app
                .global_shortcut()
                .unregister(preferences.hotkey.as_str());
        }
        if previous.startup {
            let _ = app.autolaunch().enable();
        } else {
            let _ = app.autolaunch().disable();
        }
        return Err(error);
    }
    if previous.hotkey != preferences.hotkey {
        let _ = app.global_shortcut().unregister(previous.hotkey.as_str());
    }
    crate::lifecycle::set_pet_visible(&app, preferences.enabled)?;
    *state.preferences.lock().map_err(|e| e.to_string())? = preferences;
    Ok(())
}
