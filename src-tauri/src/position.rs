use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SavedPosition {
    pub x: i32,
    pub y: i32,
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
    pub interacting: bool,
    pub roam_generation: u64,
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
        });
    let x = position.x.clamp(
        area.position.x,
        (area.position.x + area.size.width as i32 - size.width as i32).max(area.position.x),
    );
    let y = position.y.clamp(
        area.position.y,
        (area.position.y + area.size.height as i32 - size.height as i32).max(area.position.y),
    );
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    update_layout(app)
}

fn update_layout(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("No display")?;
    let area = monitor.work_area();
    let at = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    let mut position = state.position.lock().map_err(|e| e.to_string())?;
    position.right = at.x + size.width as i32 / 2 > area.position.x + area.size.width as i32 / 2;
    position.top = at.y + size.height as i32 / 2 < area.position.y + area.size.height as i32 / 2;
    Ok(())
}

#[tauri::command]
pub fn drag_start(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    state.position.lock().map_err(|e| e.to_string())?.drag = Some((
        window.cursor_position().map_err(|e| e.to_string())?,
        window.outer_position().map_err(|e| e.to_string())?,
    ));
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

pub fn snapped(value: i32, min: i32, max: i32, threshold: i32) -> i32 {
    let value = value.clamp(min, max.max(min));
    if value - min < threshold {
        min
    } else if max - value < threshold {
        max.max(min)
    } else {
        value
    }
}

#[tauri::command]
pub fn drag_end(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    state.position.lock().map_err(|e| e.to_string())?.drag = None;
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("No display")?;
    let area = monitor.work_area();
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let at = window.outer_position().map_err(|e| e.to_string())?;
    let threshold = (24.0 * monitor.scale_factor()) as i32;
    let at = PhysicalPosition::new(
        snapped(
            at.x,
            area.position.x,
            area.position.x + area.size.width as i32 - size.width as i32,
            threshold,
        ),
        snapped(
            at.y,
            area.position.y,
            area.position.y + area.size.height as i32 - size.height as i32,
            threshold,
        ),
    );
    window.set_position(at).map_err(|e| e.to_string())?;
    {
        let mut position = state.position.lock().map_err(|e| e.to_string())?;
        let key = monitor_key(&monitor);
        position
            .saved
            .monitors
            .insert(key.clone(), SavedPosition { x: at.x, y: at.y });
        position.saved.last_monitor = Some(key);
        let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(
            dir.join("positions.json"),
            serde_json::to_vec_pretty(&position.saved).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    update_layout(&app)
}

#[tauri::command]
pub fn hit_regions(state: State<AppState>, regions: Vec<HitRegion>) {
    if let Ok(mut position) = state.position.lock() {
        position.regions = regions.into_iter().take(30).collect();
    }
}

#[tauri::command]
pub fn set_interacting(state: State<AppState>, interacting: bool) {
    if let Ok(mut position) = state.position.lock() {
        position.interacting = interacting;
        if interacting {
            position.roam_generation += 1;
        }
    }
}

#[tauri::command]
pub fn roam(app: AppHandle, state: State<AppState>, direction: i32) -> Result<i32, String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("No display")?;
    let area = monitor.work_area();
    let origin = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let mut position = state.position.lock().map_err(|e| e.to_string())?;
    if position.interacting || position.drag.is_some() {
        return Ok(0);
    }
    let home = position
        .saved
        .monitors
        .get(&monitor_key(&monitor))
        .map(|p| p.x)
        .unwrap_or(area.position.x + 8);
    let limit = (100.0 * monitor.scale_factor()) as i32;
    let target = (origin.x + direction.signum() * (32.0 * monitor.scale_factor()) as i32)
        .clamp(home - limit, home + limit)
        .clamp(
            area.position.x,
            (area.position.x + area.size.width as i32 - size.width as i32).max(area.position.x),
        );
    let delta = target - origin.x;
    position.roam_generation += 1;
    let generation = position.roam_generation;
    drop(position);
    std::thread::spawn(move || {
        for step in 1..=40 {
            std::thread::sleep(std::time::Duration::from_millis(30));
            let state = app.state::<AppState>();
            let position = state.position.lock().unwrap();
            if position.roam_generation != generation || position.drag.is_some() {
                break;
            }
            drop(position);
            let _ = window.set_position(PhysicalPosition::new(
                origin.x + delta * step / 40,
                origin.y,
            ));
        }
    });
    Ok(delta)
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
    fn snapping_handles_negative_monitors_and_small_displays() {
        assert_eq!(snapped(-1910, -1920, -350, 24), -1920);
        assert_eq!(snapped(-360, -1920, -350, 24), -350);
        assert_eq!(snapped(500, 0, -10, 24), 0);
    }
}
