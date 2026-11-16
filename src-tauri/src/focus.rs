use crate::{preferences::Preferences, AppState};
use tauri::{AppHandle, Manager};

pub fn parse_hm(value: &str) -> Option<u16> {
    let (hours, minutes) = value.split_once(':')?;
    let (hours, minutes) = (hours.parse::<u16>().ok()?, minutes.parse::<u16>().ok()?);
    (hours < 24 && minutes < 60 && value.len() == 5).then_some(hours * 60 + minutes)
}

fn in_window(now: u16, start: u16, end: u16) -> bool {
    if start <= end {
        (start..end).contains(&now)
    } else {
        now >= start || now < end
    }
}

#[cfg(windows)]
fn local_minutes() -> Option<u16> {
    let mut time = unsafe { std::mem::zeroed() };
    unsafe { windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut time) };
    Some(time.wHour * 60 + time.wMinute)
}

#[cfg(not(windows))]
fn local_minutes() -> Option<u16> {
    let output = std::process::Command::new("date")
        .arg("+%H:%M")
        .output()
        .ok()?;
    parse_hm(String::from_utf8_lossy(&output.stdout).trim())
}

#[cfg(windows)]
fn focus_assist_on() -> bool {
    #[link(name = "ntdll")]
    extern "system" {
        fn NtQueryWnfStateData(
            state_name: *const u64,
            type_id: *const core::ffi::c_void,
            scope: *const core::ffi::c_void,
            change_stamp: *mut u32,
            buffer: *mut core::ffi::c_void,
            buffer_size: *mut u32,
        ) -> i32;
    }
    // WNF_SHEL_QUIETHOURS_ACTIVE_PROFILE_CHANGED: 0 off, 1 priority only, 2 alarms only.
    const STATE: u64 = 0x0D83_063E_A3BF_1C75;
    let (mut stamp, mut profile, mut size) = (0u32, 0u32, 4u32);
    let status = unsafe {
        NtQueryWnfStateData(
            &STATE,
            std::ptr::null(),
            std::ptr::null(),
            &mut stamp,
            (&mut profile as *mut u32).cast(),
            &mut size,
        )
    };
    status >= 0 && profile != 0
}

#[cfg(not(windows))]
fn focus_assist_on() -> bool {
    false
}

pub fn quiet(preferences: &Preferences) -> bool {
    let scheduled = preferences.quiet_hours
        && match (
            parse_hm(&preferences.quiet_start),
            parse_hm(&preferences.quiet_end),
            local_minutes(),
        ) {
            (Some(start), Some(end), Some(now)) => in_window(now, start, end),
            _ => false,
        };
    scheduled || (preferences.follow_focus_assist && focus_assist_on())
}

#[cfg(windows)]
fn fullscreen_app(pet: &tauri::WebviewWindow) -> bool {
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
        UI::WindowsAndMessaging::{
            GetClassNameW, GetForegroundWindow, GetShellWindow, GetWindowLongW, GetWindowRect,
            GetWindowThreadProcessId, GWL_STYLE, WS_CAPTION,
        },
    };
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.is_null() || foreground == GetShellWindow() {
            return false;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(foreground, &mut pid);
        if pid == std::process::id() {
            return false;
        }
        let mut class = [0u16; 64];
        let length = GetClassNameW(foreground, class.as_mut_ptr(), class.len() as i32);
        let class = String::from_utf16_lossy(&class[..length.max(0) as usize]);
        if matches!(class.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd") {
            return false;
        }
        // Captioned windows are maximized, not fullscreen, even when they cover the monitor.
        if GetWindowLongW(foreground, GWL_STYLE) as u32 & WS_CAPTION == WS_CAPTION {
            return false;
        }
        let monitor = MonitorFromWindow(foreground, MONITOR_DEFAULTTONEAREST);
        if let Ok(hwnd) = pet.hwnd() {
            if MonitorFromWindow(hwnd.0 as _, MONITOR_DEFAULTTONEAREST) != monitor {
                return false;
            }
        }
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        let mut rect: RECT = std::mem::zeroed();
        if GetMonitorInfoW(monitor, &mut info) == 0 || GetWindowRect(foreground, &mut rect) == 0 {
            return false;
        }
        let screen = info.rcMonitor;
        rect.left <= screen.left
            && rect.top <= screen.top
            && rect.right >= screen.right
            && rect.bottom >= screen.bottom
    }
}

#[cfg(not(windows))]
fn fullscreen_app(_: &tauri::WebviewWindow) -> bool {
    false
}

pub fn start_fullscreen_watch(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let Some(window) = app.get_webview_window("pet") else {
            break;
        };
        let state = app.state::<AppState>();
        let preferences = state.preferences.lock().unwrap().clone();
        let hidden = state.lifecycle.lock().unwrap().fullscreen_hidden;
        let want = preferences.enabled
            && preferences.hide_in_fullscreen
            && !preferences.ghost
            && fullscreen_app(&window);
        if want && !hidden && window.is_visible().unwrap_or(false) {
            state.lifecycle.lock().unwrap().fullscreen_hidden = true;
            let _ = window.hide();
        } else if !want && hidden {
            state.lifecycle.lock().unwrap().fullscreen_hidden = false;
            if preferences.enabled {
                let _ = crate::lifecycle::set_pet_visible(&app, true);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quiet_hours_parse_and_wrap_past_midnight() {
        assert_eq!(parse_hm("22:00"), Some(1320));
        assert_eq!(parse_hm("07:05"), Some(425));
        for bad in ["24:00", "7:05", "12:60", "", "ab:cd"] {
            assert_eq!(parse_hm(bad), None, "{bad}");
        }
        assert!(in_window(1380, 1320, 480));
        assert!(in_window(60, 1320, 480));
        assert!(!in_window(480, 1320, 480));
        assert!(in_window(720, 540, 1020));
        assert!(!in_window(1020, 540, 1020));
        assert!(!in_window(100, 600, 600));
    }
}
