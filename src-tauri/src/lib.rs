mod activity;
mod bridge;

use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    activity: Mutex<activity::Activity>,
    bridge_error: Mutex<Option<String>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    sessions: Vec<activity::Session>,
    hooks_installed: bool,
    bridge_error: Option<String>,
}

#[tauri::command]
fn snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    Ok(Snapshot {
        sessions: state.activity.lock().map_err(|e| e.to_string())?.visible(bridge::now()),
        hooks_installed: bridge::installed(),
        bridge_error: state.bridge_error.lock().map_err(|e| e.to_string())?.clone(),
    })
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            if let Err(error) = bridge::start(app.handle().clone()) {
                *app.state::<AppState>().bridge_error.lock().unwrap() = Some(error);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![snapshot, bridge::configure_hooks])
        .run(tauri::generate_context!())
        .expect("Crawde could not start");
}
