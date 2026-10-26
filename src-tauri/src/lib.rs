mod activity;
mod bridge;
mod position;

use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    activity: Mutex<activity::Activity>,
    bridge_error: Mutex<Option<String>>,
    position: Mutex<position::PositionState>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    sessions: Vec<activity::Session>,
    hooks_installed: bool,
    bridge_error: Option<String>,
    dock_right: bool,
    dock_top: bool,
}

#[tauri::command]
fn snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    Ok(Snapshot {
        sessions: state.activity.lock().map_err(|e| e.to_string())?.visible(bridge::now()),
        hooks_installed: bridge::installed(),
        bridge_error: state.bridge_error.lock().map_err(|e| e.to_string())?.clone(),
        dock_right: state.position.lock().map_err(|e| e.to_string())?.right,
        dock_top: state.position.lock().map_err(|e| e.to_string())?.top,
    })
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            if let Err(error) = bridge::start(app.handle().clone()) {
                *app.state::<AppState>().bridge_error.lock().unwrap() = Some(error);
            }
            position::initialize(app.handle())?;
            position::start_hit_testing(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![snapshot, bridge::configure_hooks, position::drag_start, position::drag_move, position::drag_end, position::hit_regions, position::set_interacting, position::roam])
        .run(tauri::generate_context!())
        .expect("Crawde could not start");
}
