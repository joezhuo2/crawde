mod activity;
mod bridge;
mod lifecycle;
mod navigation;
mod notifications;
mod position;
mod preferences;
mod titles;
mod updates;

use std::sync::Mutex;
use tauri::{Manager, State};

#[derive(Default)]
struct AppState {
    activity: Mutex<activity::Activity>,
    bridge_error: Mutex<Option<String>>,
    position: Mutex<position::PositionState>,
    preferences: Mutex<preferences::Preferences>,
    lifecycle: Mutex<lifecycle::Lifecycle>,
    notifications: Mutex<notifications::Notifications>,
    navigation: Mutex<navigation::Navigation>,
    update: Mutex<Option<updates::UpdateInfo>>,
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
    last_prompt_at: u64,
    last_activity_at: u64,
    last_event_at: u64,
    update: Option<updates::UpdateInfo>,
}

#[tauri::command]
fn snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    let (dock_right, dock_top) = {
        let position = state.position.lock().map_err(|e| e.to_string())?;
        (position.right, position.top)
    };
    let (sessions, last_prompt_at, last_activity_at, last_event_at, recent_work) = {
        let mut activity = state.activity.lock().map_err(|e| e.to_string())?;
        (activity.visible(bridge::now()), activity.last_prompt_at, activity.last_activity_at, activity.last_event_at, activity.has_recent_work(bridge::now()))
    };
    Ok(Snapshot {
        sessions,
        last_prompt_at,
        last_activity_at,
        last_event_at,
        update: state.update.lock().map_err(|e| e.to_string())?.clone(),
        hooks_installed: bridge::installed(),
        bridge_error: state
            .bridge_error
            .lock()
            .map_err(|e| e.to_string())?
            .clone(),
        dock_right,
        dock_top,
        claude_open: state
            .lifecycle
            .lock()
            .map_err(|e| e.to_string())?
            .claude_open || recent_work,
        preferences: state.preferences.lock().map_err(|e| e.to_string())?.clone(),
    })
}

#[tauri::command]
fn dismiss_chat(state: State<AppState>, id: String) -> Result<(), String> {
    state
        .activity
        .lock()
        .map_err(|e| e.to_string())?
        .dismiss(&id);
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if args.iter().any(|arg| arg == "--toggle") {
                let _ = lifecycle::toggle(app);
            } else if !args.iter().any(|arg| arg == "--background") {
                let _ = lifecycle::set_pet_visible(app, true);
                let _ = lifecycle::show_settings(app.clone());
            }
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--background"])
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .setup(|app| {
            if let Err(error) = bridge::start(app.handle().clone()) {
                *app.state::<AppState>().bridge_error.lock().unwrap() = Some(error);
            }
            position::initialize(app.handle())?;
            position::start_hit_testing(app.handle().clone());
            lifecycle::setup(app.handle())?;
            updates::check_on_open(app.handle());
            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "settings" {
                let _ = lifecycle::show_settings(app.clone());
            }
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            dismiss_chat,
            bridge::configure_hooks,
            position::drag_start,
            position::drag_move,
            position::drag_end,
            position::hit_regions,
            position::set_interacting,
            position::roam,
            lifecycle::show_settings,
            lifecycle::show_context_menu,
            lifecycle::quit,
            lifecycle::close_settings,
            lifecycle::set_visibility,
            preferences::save_preferences,
            notifications::test_sound,
            navigation::open_chat,
            navigation::new_chat,
            updates::check_updates,
            updates::open_releases
        ])
        .run(tauri::generate_context!())
        .expect("Crawde could not start");
}
