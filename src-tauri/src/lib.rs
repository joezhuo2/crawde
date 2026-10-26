mod activity;
mod bridge;
mod position;
mod preferences;
mod lifecycle;

use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    activity: Mutex<activity::Activity>,
    bridge_error: Mutex<Option<String>>,
    position: Mutex<position::PositionState>,
    preferences: Mutex<preferences::Preferences>,
    lifecycle: Mutex<lifecycle::Lifecycle>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    sessions: Vec<activity::Session>,
    hooks_installed: bool,
    bridge_error: Option<String>,
    dock_right: bool,
    dock_top: bool,
    claude_open: bool,
    preferences: preferences::Preferences,
}

#[tauri::command]
fn snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    Ok(Snapshot {
        sessions: state.activity.lock().map_err(|e| e.to_string())?.visible(bridge::now()),
        hooks_installed: bridge::installed(),
        bridge_error: state.bridge_error.lock().map_err(|e| e.to_string())?.clone(),
        dock_right: state.position.lock().map_err(|e| e.to_string())?.right,
        dock_top: state.position.lock().map_err(|e| e.to_string())?.top,
        claude_open: state.lifecycle.lock().map_err(|e| e.to_string())?.claude_open,
        preferences: state.preferences.lock().map_err(|e| e.to_string())?.clone(),
    })
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if !args.iter().any(|arg| arg == "--background") { let _ = lifecycle::show_settings(app.clone()); }
        }))
        .plugin(tauri_plugin_autostart::Builder::new().args(["--background"]).build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::default())
        .setup(|app| {
            if let Err(error) = bridge::start(app.handle().clone()) {
                *app.state::<AppState>().bridge_error.lock().unwrap() = Some(error);
            }
            position::initialize(app.handle())?;
            position::start_hit_testing(app.handle().clone());
            lifecycle::setup(app.handle())?;
            Ok(())
        })
        .on_menu_event(|app, event| { if event.id.as_ref() == "settings" { let _ = lifecycle::show_settings(app.clone()); } })
        .invoke_handler(tauri::generate_handler![snapshot, bridge::configure_hooks, position::drag_start, position::drag_move, position::drag_end, position::hit_regions, position::set_interacting, position::roam, lifecycle::show_settings, lifecycle::show_context_menu, lifecycle::quit])
        .run(tauri::generate_context!())
        .expect("Crawde could not start");
}
