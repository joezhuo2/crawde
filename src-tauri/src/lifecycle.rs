use crate::{preferences, AppState};
use tauri::{
    menu::{CheckMenuItem, ContextMenu, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Default)]
pub struct Lifecycle {
    pub claude_open: bool,
}

#[tauri::command]
pub fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show().map_err(|e| e.to_string())?;
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("index.html?settings".into()),
    )
    .title("Crawde · Settings")
    .inner_size(430.0, 650.0)
    .min_inner_size(400.0, 500.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    let state = app.state::<AppState>();
    let enabled = !window.is_visible().map_err(|e| e.to_string())?;
    set_pet_visible(app, enabled)?;
    let updated = {
        let mut prefs = state.preferences.lock().map_err(|e| e.to_string())?;
        prefs.enabled = enabled;
        prefs.clone()
    };
    preferences::persist(app, &updated)
}

pub fn set_pet_visible(app: &AppHandle, visible: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    if visible {
        crate::position::restore(app)?;
        window.unminimize().map_err(|e| e.to_string())?;
        window.set_always_on_top(true).map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())
    } else {
        window.hide().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn set_visibility(app: AppHandle, visible: bool) -> Result<(), String> {
    set_pet_visible(&app, visible)?;
    let updated = {
        let state = app.state::<AppState>();
        let mut prefs = state.preferences.lock().map_err(|e| e.to_string())?;
        prefs.enabled = visible;
        prefs.clone()
    };
    preferences::persist(&app, &updated)
}

pub fn context_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let busy = app
        .state::<AppState>()
        .update_busy
        .load(std::sync::atomic::Ordering::SeqCst);
    let update = app.state::<AppState>().update.lock().unwrap().clone();
    let (label, enabled) = crate::updates::menu_label(update.as_ref(), busy);
    let update_action = MenuItem::with_id(app, "update", label, enabled, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Crawde", true, None::<&str>)?;
    let prefs = app.state::<AppState>().preferences.lock().unwrap().clone();
    let motion = CheckMenuItem::with_id(
        app,
        "reducedMotion",
        "Reduce motion",
        true,
        prefs.reduced_motion,
        None::<&str>,
    )?;
    let startup = CheckMenuItem::with_id(
        app,
        "startup",
        "Launch on startup",
        true,
        prefs.startup,
        None::<&str>,
    )?;
    let walks = CheckMenuItem::with_id(
        app,
        "roaming",
        "Little walks",
        true,
        prefs.roaming,
        None::<&str>,
    )?;
    let sound = CheckMenuItem::with_id(
        app,
        "sound",
        "Sound effects",
        true,
        prefs.sound,
        None::<&str>,
    )?;
    let installed = crate::bridge::installed();
    let hooks = MenuItem::with_id(
        app,
        "install_hooks",
        if installed {
            "Reinstall activity hooks"
        } else {
            "Install activity hooks"
        },
        true,
        None::<&str>,
    )?;
    let disconnect = MenuItem::with_id(
        app,
        "disconnect_hooks",
        "Disconnect activity hooks",
        installed,
        None::<&str>,
    )?;
    let menu = Menu::with_items(
        app,
        &[
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &motion,
            &startup,
            &walks,
            &sound,
            &PredefinedMenuItem::separator(app)?,
            &hooks,
            &disconnect,
            &PredefinedMenuItem::separator(app)?,
            &update_action,
            &quit,
        ],
    )?;
    let message = app.state::<AppState>().menu_message.lock().unwrap().clone();
    if let Some(message) = message {
        menu.append(&MenuItem::with_id(
            app,
            "menu_status",
            message,
            false,
            None::<&str>,
        )?)?;
    }
    if let Some(info) = update.filter(|info| !info.check_succeeded && !busy) {
        menu.append(&MenuItem::with_id(
            app,
            "update_status",
            &info.message,
            false,
            None::<&str>,
        )?)?;
    }
    Ok(menu)
}

pub fn refresh_menu(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("crawde"), context_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

pub fn handle_menu(app: &AppHandle, id: &str) {
    let result = match id {
        "settings" => show_settings(app.clone()),
        "quit" => {
            quit(app.clone());
            return;
        }
        "update" => {
            crate::updates::activate(app);
            return;
        }
        "reducedMotion" | "startup" | "roaming" | "sound" => {
            let prefs = app.state::<AppState>().preferences.lock().unwrap().clone();
            let current = match id {
                "reducedMotion" => prefs.reduced_motion,
                "startup" => prefs.startup,
                "roaming" => prefs.roaming,
                _ => prefs.sound,
            };
            preferences::update_preference(app.clone(), id.into(), serde_json::json!(!current))
        }
        "install_hooks" | "disconnect_hooks" => {
            crate::bridge::configure_hooks(app.clone(), id == "install_hooks")
        }
        _ => return,
    };
    let message = match result {
        Err(error) => Some(error),
        Ok(()) if id == "install_hooks" => {
            Some("Hooks installed. Restart existing Claude sessions.".into())
        }
        Ok(()) => None,
    };
    *app.state::<AppState>().menu_message.lock().unwrap() = message;
    refresh_menu(app);
}

#[tauri::command]
pub fn show_context_menu(app: AppHandle) -> Result<(), String> {
    let menu = context_menu(&app).map_err(|e| e.to_string())?;
    let window = app
        .get_webview_window("pet")
        .ok_or("Pet window unavailable")?;
    menu.popup(window.as_ref().window())
        .map_err(|e| e.to_string())
}

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = preferences::load(app);
    *app.state::<AppState>().preferences.lock().unwrap() = prefs.clone();
    if crate::bridge::installed() && crate::bridge::needs_repair() {
        if let Err(error) = crate::bridge::configure_hooks(app.clone(), true) {
            *app.state::<AppState>().bridge_error.lock().unwrap() =
                Some(format!("Activity hooks need reconnecting: {error}"));
        }
    }
    if prefs.startup && !cfg!(debug_assertions) {
        app.autolaunch().enable()?;
    }
    app.global_shortcut()
        .on_shortcut(prefs.hotkey.as_str(), |app, _, event| {
            if event.state == ShortcutState::Pressed {
                let _ = toggle(app);
            }
        })?;
    let menu = context_menu(app)?;
    TrayIconBuilder::with_id("crawde")
        .icon(tauri::image::Image::from_bytes(include_bytes!(
            "../icons/tray.png"
        ))?)
        .tooltip("Crawde · click to show or hide")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let _ = toggle(tray.app_handle());
            }
        })
        .build(app)?;
    if !crate::bridge::installed() {
        show_settings(app.clone())?;
    }
    // Interactive launches show a sleeping pet even if Claude is not running.
    // Only Windows startup launches wait quietly in the tray.
    if prefs.enabled && !std::env::args().any(|arg| arg == "--background") {
        set_pet_visible(app, true)?;
    }
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
            let enabled = state.preferences.lock().unwrap().enabled;
            if open && !was_open && enabled {
                let _ = set_pet_visible(&app, true);
            }
            // Keep the pet sleeping when Claude closes. Recheck monitor topology for docking/reconnection.
            if let Ok(monitors) = app.available_monitors() {
                let keys = format!("{monitors:?}");
                if keys != display_keys {
                    let _ = crate::position::restore(&app);
                    display_keys = keys;
                }
            }
            crate::notifications::tick(&app);
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    });
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn close_settings(app: AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.close();
    }
}
