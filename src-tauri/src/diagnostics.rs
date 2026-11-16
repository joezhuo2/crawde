use crate::AppState;
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct Check {
    name: String,
    status: &'static str,
    detail: String,
}

fn check(name: &str, status: &'static str, detail: impl Into<String>) -> Check {
    Check {
        name: name.into(),
        status,
        detail: crate::log::redact(&detail.into()),
    }
}

fn bridge(app: &AppHandle) -> Check {
    if let Some(error) = app.state::<AppState>().bridge_error.lock().unwrap().clone() {
        return check("Activity bridge", "fail", error);
    }
    let token = match crate::bridge::token(app) {
        Ok(token) => token,
        Err(error) => {
            return check(
                "Activity bridge",
                "fail",
                format!("No access token: {error}"),
            )
        }
    };
    // An authorized empty event is rejected as malformed, proving the receiver and token without creating a card.
    match crate::bridge::post(&token, b"{}") {
        Ok(response) if response.starts_with("HTTP/1.1 400") => check(
            "Activity bridge",
            "pass",
            "Listening on 127.0.0.1:47631 and accepting the access token",
        ),
        Ok(response) if response.starts_with("HTTP/1.1 403") => check(
            "Activity bridge",
            "fail",
            "The receiver rejected Crawde's own access token",
        ),
        Ok(response) => check(
            "Activity bridge",
            "fail",
            format!(
                "Unexpected reply: {}",
                response.lines().next().unwrap_or("")
            ),
        ),
        Err(error) => check(
            "Activity bridge",
            "fail",
            format!("Could not connect: {error}"),
        ),
    }
}

fn settings() -> Check {
    let path = crate::bridge::config_dir().join("settings.json");
    match std::fs::read_to_string(&path) {
        Err(_) => check(
            "Claude settings",
            "warn",
            format!("{} was not found", path.display()),
        ),
        Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(_) => check("Claude settings", "pass", path.display().to_string()),
            Err(error) => check(
                "Claude settings",
                "fail",
                format!("{} is not valid JSON: {error}", path.display()),
            ),
        },
    }
}

fn hooks(app: &AppHandle) -> Check {
    if !crate::bridge::installed() {
        return check(
            "Activity hooks",
            "fail",
            "Not installed. Right-click Crawde and choose Install activity hooks.",
        );
    }
    match crate::bridge::stale_reason(app) {
        Some(reason) => check(
            "Activity hooks",
            "fail",
            format!("Reinstall needed: {reason}"),
        ),
        None => check("Activity hooks", "pass", "Installed and up to date"),
    }
}

fn activity(app: &AppHandle) -> Check {
    let last = app
        .state::<AppState>()
        .activity
        .lock()
        .unwrap()
        .last_event_at;
    if last == 0 {
        check(
            "Recent activity",
            "warn",
            "No events received yet. Restart Claude sessions after installing hooks.",
        )
    } else {
        check(
            "Recent activity",
            "pass",
            format!(
                "Last event {}",
                crate::lifecycle::ago(crate::bridge::now().saturating_sub(last))
            ),
        )
    }
}

fn statusline() -> Check {
    let command = crate::bridge::read_settings()
        .and_then(|s| {
            s.get("statusLine")?
                .get("command")?
                .as_str()
                .map(String::from)
        })
        .unwrap_or_default();
    if !command.contains("crawde-statusline") {
        return check(
            "Status line",
            "warn",
            "Not set up. Optional; needed for plan usage rings and exact context %.",
        );
    }
    let recent = std::fs::read_dir(crate::platform::home_dir().join(".crawde/statusline"))
        .ok()
        .and_then(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.metadata().ok()?.modified().ok())
                .max()
        });
    match recent {
        Some(_) => check("Status line", "pass", "Configured and writing data"),
        None => check(
            "Status line",
            "warn",
            "Configured, but no data yet. It writes after Claude's next response.",
        ),
    }
}

#[cfg(windows)]
fn wsl() -> Vec<Check> {
    let Some(distros) = crate::wsl::distros() else {
        return vec![check("WSL", "pass", "Not installed; skipped")];
    };
    if distros.is_empty() {
        return vec![check("WSL", "pass", "No distros; skipped")];
    }
    distros
        .iter()
        .map(|distro| {
            let name = format!("WSL · {distro}");
            if crate::wsl::valid_distro(distro)
                && !std::path::Path::new(&format!(r"\\wsl.localhost\{distro}\")).exists()
            {
                return check(&name, "fail", format!(r"\\wsl.localhost\{distro} is not reachable"));
            }
            let mut command = crate::wsl::quiet_command();
            command.args([
                "--distribution",
                distro,
                "--exec",
                "sh",
                "-c",
                r#"for t in python3 wslpath; do command -v $t >/dev/null || echo "missing $t"; done; grep -q -- --crawde-hook "${CLAUDE_CONFIG_DIR:-$HOME/.claude}/settings.json" 2>/dev/null || echo "hooks not installed""#,
            ]);
            match crate::wsl::output(command, None) {
                Err(error) => check(&name, "fail", error),
                Ok(bytes) => {
                    let problems = String::from_utf8_lossy(&bytes).trim().replace('\n', ", ");
                    if problems.is_empty() {
                        check(&name, "pass", "python3, wslpath, and hooks are present")
                    } else {
                        check(&name, "fail", problems)
                    }
                }
            }
        })
        .collect()
}

#[cfg(not(windows))]
fn wsl() -> Vec<Check> {
    Vec::new()
}

#[tauri::command]
pub async fn run_diagnostics(app: AppHandle) -> Vec<Check> {
    let mut checks = vec![
        bridge(&app),
        settings(),
        hooks(&app),
        activity(&app),
        statusline(),
    ];
    checks.extend(wsl());
    let failed: Vec<&str> = checks
        .iter()
        .filter(|c| c.status == "fail")
        .map(|c| c.name.as_str())
        .collect();
    crate::log::record(
        "INFO",
        format!(
            "Diagnostics: {} checks, failed: {}",
            checks.len(),
            if failed.is_empty() {
                "none".into()
            } else {
                failed.join(", ")
            }
        ),
    );
    checks
}
