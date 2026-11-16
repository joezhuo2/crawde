use std::{io::Write, path::PathBuf};
use tauri::AppHandle;

const MAX_BYTES: u64 = 512 * 1024;

fn path() -> Option<PathBuf> {
    Some(crate::platform::app_data_dir()?.join("com.joezhuo.crawde/crawde.log"))
}

// Civil-from-days (Howard Hinnant) keeps timestamps readable without a date dependency.
fn timestamp(ms: u64) -> String {
    let (days, rest) = ((ms / 86_400_000) as i64, ms % 86_400_000);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}Z",
        rest / 3_600_000,
        rest / 60_000 % 60,
        rest / 1000 % 60
    )
}

pub fn record(level: &str, message: impl AsRef<str>) {
    let Some(path) = path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES) {
        let _ = std::fs::rename(&path, path.with_extension("log.old"));
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let message: String = message.as_ref().chars().take(2000).collect();
        let _ = writeln!(
            file,
            "{} [{level}] {}",
            timestamp(crate::bridge::now()),
            message.replace('\n', " | ")
        );
    }
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        record("PANIC", info.to_string());
        previous(info);
    }));
}

pub fn redact(text: &str) -> String {
    let home = crate::platform::home_dir();
    let home = home.to_string_lossy();
    if home.len() < 3 {
        return text.into();
    }
    text.replace(home.as_ref(), "~")
        .replace(&home.replace('\\', "/"), "~")
}

#[tauri::command]
pub fn read_logs() -> String {
    let Some(path) = path() else {
        return String::new();
    };
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    redact(&lines[lines.len().saturating_sub(300)..].join("\n"))
}

#[tauri::command]
pub fn log_message(message: String) {
    record("UI", message);
}

#[tauri::command]
pub fn open_log_folder(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let path = path().ok_or("Log folder is unavailable")?;
    record("INFO", "Opened log folder");
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timestamps_are_utc_calendar_dates() {
        assert_eq!(timestamp(0), "1970-01-01 00:00:00Z");
        assert_eq!(timestamp(951_782_400_000), "2000-02-29 00:00:00Z");
        assert_eq!(timestamp(1_794_844_041_000), "2026-11-16 15:47:21Z");
    }
    #[test]
    fn redaction_hides_the_home_folder() {
        let home = crate::platform::home_dir().to_string_lossy().into_owned();
        if home.len() >= 3 {
            assert_eq!(redact(&format!("{home}/x")), "~/x");
        }
    }
}
