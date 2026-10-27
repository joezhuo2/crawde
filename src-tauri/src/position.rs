use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SavedPosition {
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub pet_x: Option<f64>,
    #[serde(default)]
    pub pet_y: Option<f64>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PositionFile {
    pub monitors: BTreeMap<String, SavedPosition>,
    pub last_monitor: Option<String>,
}
#[derive(Clone, Deserialize)]
pub struct HitRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Default)]
pub struct PositionState {
    pub saved: PositionFile,
    pub drag: Option<(PhysicalPosition<f64>, PhysicalPosition<i32>)>,
    pub regions: Vec<HitRegion>,
    pub right: bool,
    pub top: bool,
    pub pet_x: Option<f64>,
    pub pet_y: Option<f64>,
}

fn monitor_key(monitor: &tauri::Monitor) -> String {
    format!(
        "{}:{}x{}@{},{}",
        monitor.name().map(String::as_str).unwrap_or("display"),
        monitor.size().width,
        monitor.size().height,
        monitor.position().x,
        monitor.position().y
    )
}

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("positions.json");
    let saved = std::fs::read(&path)
        .ok()
        .and_then(|s| serde_json::from_slice::<PositionFile>(&s).ok())
        .unwrap_or_default();
    app.state::<AppState>().position.lock().unwrap().saved = saved;
    restore(app)
}

pub fn restore(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    let saved = state
        .position
        .lock()
        .map_err(|e| e.to_string())?
        .saved
        .clone();
    let monitor = monitors
        .iter()
        .find(|m| Some(monitor_key(m)) == saved.last_monitor)
        .or(monitors.first())
        .ok_or("No display available")?;
    let area = monitor.work_area();
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let position = saved
        .monitors
        .get(&monitor_key(monitor))
        .cloned()
        .unwrap_or(SavedPosition {
            x: area.position.x + 8,
            y: area.position.y + area.size.height as i32 - size.height as i32 - 8,
            pet_x: None,
            pet_y: None,
        });
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let pet_scale = state.preferences.lock().map_err(|e| e.to_string())?.scale as f64;
    // Old position files stored only the window origin. Keep their visible crab
    // location on first migration, then persist its independent local anchor.
    let old_top =
        position.y + size.height as i32 / 2 < area.position.y + area.size.height as i32 / 2;
    let old_right =
        position.x + size.width as i32 / 2 > area.position.x + area.size.width as i32 / 2;
    let pet = HitRegion {
        x: position.pet_x.unwrap_or(if old_right {
            size.width as f64 / scale - 16.0 - 32.0 * pet_scale
        } else {
            16.0
        }),
        y: position.pet_y.unwrap_or(if old_top {
            8.0
        } else {
            size.height as f64 / scale - 6.0 - 24.0 * pet_scale
        }),
        width: 32.0 * pet_scale,
        height: 24.0 * pet_scale,
    };
    place_pet(
        app,
        PhysicalPosition::new(position.x, position.y),
        &pet,
        monitor,
    )
}

// Preserve the screen-space pet coordinate while relocating the transparent
// window around it. Only the pet itself is constrained to the work area.
fn place_axis(
    pet: f64,
    pet_size: f64,
    preferred_window: f64,
    window_size: f64,
    min: f64,
    length: f64,
) -> (i32, f64) {
    let pet = pet.clamp(min, (min + length - pet_size).max(min));
    let window = preferred_window
        .clamp(min, (min + length - window_size).max(min))
        .round() as i32;
    (window, pet - window as f64)
}

fn place_pet(
    app: &AppHandle,
    origin: PhysicalPosition<i32>,
    pet: &HitRegion,
    monitor: &tauri::Monitor,
) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let area = monitor.work_area();
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let screen_x = origin.x as f64 + pet.x * scale;
    let screen_y = origin.y as f64 + pet.y * scale;
    let top = screen_y + pet.height * scale / 2.0
        < area.position.y as f64 + area.size.height as f64 / 2.0;
    let preferred_y = if top {
        screen_y - 8.0 * scale
    } else {
        screen_y + (pet.height + 6.0) * scale - size.height as f64
    };
    let (x, pet_x) = place_axis(
        screen_x,
        pet.width * scale,
        origin.x as f64,
        size.width as f64,
        area.position.x as f64,
        area.size.width as f64,
    );
    let (y, pet_y) = place_axis(
        screen_y,
        pet.height * scale,
        preferred_y,
        size.height as f64,
        area.position.y as f64,
        area.size.height as f64,
    );
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    let mut position = state.position.lock().map_err(|e| e.to_string())?;
    position.right =
        screen_x + pet.width * scale / 2.0 > area.position.x as f64 + area.size.width as f64 / 2.0;
    position.top = top;
    position.pet_x = Some(pet_x / scale);
    position.pet_y = Some(pet_y / scale);
    Ok(())
}

#[tauri::command]
pub fn drag_start(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let drag = (
        window.cursor_position().map_err(|e| e.to_string())?,
        window.outer_position().map_err(|e| e.to_string())?,
    );
    state.position.lock().map_err(|e| e.to_string())?.drag = Some(drag);
    Ok(())
}

#[tauri::command]
pub fn drag_move(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let drag = state.position.lock().map_err(|e| e.to_string())?.drag;
    if let Some((start, origin)) = drag {
        let cursor = window.cursor_position().map_err(|e| e.to_string())?;
        window
            .set_position(PhysicalPosition::new(
                origin.x + (cursor.x - start.x) as i32,
                origin.y + (cursor.y - start.y) as i32,
            ))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn drag_end(
    app: AppHandle,
    state: State<AppState>,
    pet: HitRegion,
) -> Result<crate::Snapshot, String> {
    state.position.lock().map_err(|e| e.to_string())?.drag = None;
    if ![pet.x, pet.y, pet.width, pet.height]
        .iter()
        .all(|v| v.is_finite())
        || pet.width <= 0.0
        || pet.height <= 0.0
    {
        return Err("Invalid pet bounds".into());
    }
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let origin = window.outer_position().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let center_x = origin.x as f64 + (pet.x + pet.width / 2.0) * scale;
    let center_y = origin.y as f64 + (pet.y + pet.height / 2.0) * scale;
    let monitor = window
        .available_monitors()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|m| {
            let at = m.position();
            let size = m.size();
            center_x >= at.x as f64
                && center_x < at.x as f64 + size.width as f64
                && center_y >= at.y as f64
                && center_y < at.y as f64 + size.height as f64
        })
        .or(window.current_monitor().map_err(|e| e.to_string())?)
        .ok_or("No display")?;
    place_pet(&app, origin, &pet, &monitor)?;
    let at = window.outer_position().map_err(|e| e.to_string())?;
    {
        let mut position = state.position.lock().map_err(|e| e.to_string())?;
        let key = monitor_key(&monitor);
        let pet_x = position.pet_x;
        let pet_y = position.pet_y;
        position.saved.monitors.insert(
            key.clone(),
            SavedPosition {
                x: at.x,
                y: at.y,
                pet_x,
                pet_y,
            },
        );
        position.saved.last_monitor = Some(key);
        let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(
            dir.join("positions.json"),
            serde_json::to_vec_pretty(&position.saved).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    crate::snapshot(state)
}

#[tauri::command]
pub fn hit_regions(state: State<AppState>, regions: Vec<HitRegion>) {
    if let Ok(mut position) = state.position.lock() {
        position.regions = regions.into_iter().take(30).collect();
    }
}

pub fn start_hit_testing(app: AppHandle) {
    std::thread::spawn(move || {
        let mut ignored = false;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(40));
            let Some(window) = app.get_webview_window("pet") else {
                break;
            };
            if !window.is_visible().unwrap_or(false) {
                continue;
            }
            let (Ok(cursor), Ok(origin), Ok(scale)) = (
                window.cursor_position(),
                window.outer_position(),
                window.scale_factor(),
            ) else {
                continue;
            };
            let x = (cursor.x - origin.x as f64) / scale;
            let y = (cursor.y - origin.y as f64) / scale;
            let state = app.state::<AppState>();
            let position = state.position.lock().unwrap();
            let inside = position.drag.is_some()
                || position.regions.iter().any(|r| {
                    x >= r.x - 6.0
                        && y >= r.y - 6.0
                        && x <= r.x + r.width + 6.0
                        && y <= r.y + r.height + 6.0
                });
            drop(position);
            if ignored == inside {
                if window.set_ignore_cursor_events(!inside).is_ok() {
                    ignored = !inside;
                    let _ = app.emit_to("pet", "pet-hover", inside);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_anchors_round_trip_and_legacy_positions_still_load() {
        let legacy: SavedPosition = serde_json::from_str(r#"{"x":24,"y":100}"#).unwrap();
        assert!(legacy.pet_x.is_none() && legacy.pet_y.is_none());
        let saved = SavedPosition {
            x: 24,
            y: 100,
            pet_x: Some(16.0),
            pet_y: Some(220.0),
        };
        let restored: SavedPosition =
            serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        assert_eq!(restored.pet_y, Some(220.0));
        assert_eq!(restored.pet_x, Some(16.0));
    }
    #[test]
    fn middle_drops_keep_pet_coordinates_when_window_moves() {
        for screen_y in [160.0, 300.0, 450.0, 540.0, 700.0, 900.0] {
            let preferred = if screen_y + 36.0 < 540.0 {
                screen_y - 8.0
            } else {
                screen_y + 78.0 - 520.0
            };
            let (window, local) = place_axis(screen_y, 72.0, preferred, 520.0, 0.0, 1080.0);
            assert_eq!(window as f64 + local, screen_y);
            assert!(local >= 0.0 && local + 72.0 <= 520.0);
        }
    }
    #[test]
    fn bounds_handle_negative_monitors_dpi_and_offscreen_pet() {
        let (window, local) = place_axis(-500.0, 108.0, -800.0, 780.0, -1080.0, 1080.0);
        assert_eq!(window as f64 + local, -500.0);
        let (window, local) = place_axis(1050.0, 72.0, 608.0, 520.0, 0.0, 1080.0);
        assert_eq!(window as f64 + local, 1008.0);
        let (window, local) = place_axis(200.0, 72.0, -242.0, 520.0, 0.0, 480.0);
        assert_eq!(window as f64 + local, 200.0);
    }
}
