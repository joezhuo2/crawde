use crate::{preferences, AppState};
use tauri::{AppHandle, Manager, menu::{Menu, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_autostart::ManagerExt;

#[derive(Default)]
pub struct Lifecycle { pub claude_open: bool }

#[tauri::command]
pub fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("index.html?settings".into()))
        .title("Crawde · Settings").inner_size(430.0, 650.0).min_inner_size(400.0, 500.0)
        .resizable(true).center().build().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("pet").ok_or("Pet window unavailable")?;
    let state = app.state::<AppState>();
    let mut prefs = state.preferences.lock().map_err(|e| e.to_string())?;
    prefs.enabled = !window.is_visible().map_err(|e| e.to_string())?;
    if prefs.enabled { window.show() } else { window.hide() }.map_err(|e| e.to_string())?;
    preferences::persist(app, &prefs)
}

#[tauri::command]
pub fn show_context_menu(app: AppHandle) -> Result<(), String> {
    let item = MenuItem::with_id(&app, "settings", "Settings…", true, None::<&str>).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(&app, &[&item]).map_err(|e| e.to_string())?;
    let window = app.get_webview_window("pet").ok_or("Pet window unavailable")?;
    menu.popup(window).map_err(|e| e.to_string())
}

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = preferences::load(app);
    *app.state::<AppState>().preferences.lock().unwrap() = prefs.clone();
    if prefs.startup { app.autolaunch().enable()?; }
    app.global_shortcut().on_shortcut(prefs.hotkey.as_str(), |app, _, event| {
        if event.state == ShortcutState::Pressed { let _ = toggle(app); }
    })?;
    let item = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&item])?;
    TrayIconBuilder::with_id("crawde")
        .icon(app.default_window_icon().cloned().ok_or("Missing tray icon")?)
        .tooltip("Crawde · click to show or hide")
        .menu(&menu).show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }) {
                let _ = toggle(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| { if event.id.as_ref() == "settings" { let _ = show_settings(app.clone()); } })
        .build(app)?;
    if !crate::bridge::installed() { show_settings(app.clone())?; }
    start_presence(app.clone());
    Ok(())
}

fn start_presence(app: AppHandle) {
    std::thread::spawn(move || {
        let mut system = sysinfo::System::new();
        let mut display_keys = String::new();
        loop {
            system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            let open = system.processes().values().any(|p| {
                let name = p.name().to_string_lossy().to_ascii_lowercase();
                name == "claude.exe" || name == "claude"
            });
            let state = app.state::<AppState>();
            let mut lifecycle = state.lifecycle.lock().unwrap();
            let was_open = lifecycle.claude_open;
            lifecycle.claude_open = open;
            drop(lifecycle);
            if open && !was_open && state.preferences.lock().unwrap().enabled {
                if let Some(window) = app.get_webview_window("pet") { let _ = window.show(); }
            }
            // Keep the pet sleeping when Claude closes. Recheck monitor topology for docking/reconnection.
            if let Ok(monitors) = app.available_monitors() {
                let keys = format!("{monitors:?}");
                if keys != display_keys { let _ = crate::position::restore(&app); display_keys = keys; }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    });
}

#[tauri::command]
pub fn quit(app: AppHandle) { app.exit(0); }

#[tauri::command]
pub fn close_settings(app: AppHandle) {
    if let Some(window) = app.get_webview_window("settings") { let _ = window.close(); }
}
