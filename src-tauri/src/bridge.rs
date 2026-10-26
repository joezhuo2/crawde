use crate::{activity::HookEvent, AppState};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

// SessionStart does not support HTTP hooks. Reuse this executable as a quiet,
// bounded stdin forwarder; this path exits before Tauri creates any windows.
pub fn forward_hook() -> Result<(), String> {
    let mut body = Vec::new();
    std::io::stdin()
        .take(65_537)
        .read_to_end(&mut body)
        .map_err(|e| e.to_string())?;
    if body.len() > 65_536 {
        return Err("Hook payload is too large".into());
    }
    let path = PathBuf::from(std::env::var_os("APPDATA").ok_or("APPDATA is unavailable")?)
        .join("com.joezhuo.crawde/bridge-token");
    let token = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if !token.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err("Invalid bridge token".into());
    }
    let address: SocketAddr = "127.0.0.1:47631".parse().expect("constant address");
    let timeout = std::time::Duration::from_millis(400);
    let mut stream = TcpStream::connect_timeout(&address, timeout).map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    let header = format!("POST /events HTTP/1.1\r\nHost: 127.0.0.1:47631\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    stream
        .write_all(header.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(&body).map_err(|e| e.to_string())?;
    let mut response = [0; 128];
    let length = stream.read(&mut response).map_err(|e| e.to_string())?;
    if !String::from_utf8_lossy(&response[..length]).starts_with("HTTP/1.1 200") {
        return Err("Hook receiver rejected event".into());
    }
    Ok(())
}
use tauri::{AppHandle, Manager};
use tiny_http::{Header, Method, Response, Server, StatusCode};

pub const ENDPOINT: &str = "http://127.0.0.1:47631/events";
const EVENTS: &[&str] = &[
    "SessionStart",
    "SessionEnd",
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionRequest",
    "Notification",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "StopFailure",
    "Elicitation",
];

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("USERPROFILE").unwrap_or_default()).join(".claude")
        })
}

pub fn token(app: &AppHandle) -> Result<String, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("bridge-token");
    if path.exists() {
        return fs::read_to_string(path).map_err(|e| e.to_string());
    }
    let token = uuid::Uuid::new_v4().to_string();
    fs::write(path, &token).map_err(|e| e.to_string())?;
    Ok(token)
}

pub fn start(app: AppHandle) -> Result<(), String> {
    let server = Server::http("127.0.0.1:47631")
        .map_err(|e| format!("Activity bridge could not bind: {e}"))?;
    let expected = format!("Bearer {}", token(&app)?);
    std::thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let authorized = request
                .headers()
                .iter()
                .any(|h| h.field.equiv("Authorization") && h.value.as_str() == expected);
            let browser_origin = request.headers().iter().any(|h| h.field.equiv("Origin"));
            let status = if !authorized || browser_origin {
                403
            } else if request.url() != "/events" || request.method() != &Method::Post {
                404
            } else if request.body_length().is_some_and(|n| n > 65_536) {
                413
            } else {
                let mut body = String::new();
                match request.as_reader().take(65_537).read_to_string(&mut body) {
                    Ok(_) if body.len() <= 65_536 => {
                        match serde_json::from_str::<HookEvent>(&body) {
                            Ok(event) if EVENTS.contains(&event.hook_event_name.as_str()) => {
                                let title = crate::titles::read_title(
                                    &event.session_id,
                                    &event.transcript_path,
                                );
                                let session_id = event.session_id.clone();
                                let state = app.state::<AppState>();
                                let status = match state.activity.lock() {
                                    Ok(mut activity) => {
                                        if activity.apply(event, now()).is_ok() {
                                            if let (Some(title), Some(session)) =
                                                (title, activity.sessions.get_mut(&session_id))
                                            {
                                                session.title = title;
                                            }
                                            200
                                        } else {
                                            400
                                        }
                                    }
                                    Err(_) => 500,
                                };
                                status
                            }
                            _ => 400,
                        }
                    }
                    _ => 413,
                }
            };
            // Observational hooks must never return a Claude permission decision.
            if status == 200 {
                crate::notifications::tick(&app);
            }
            let response = Response::from_string("{}")
                .with_status_code(StatusCode(status))
                .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
            let _ = request.respond(response);
        }
    });
    Ok(())
}

fn is_our_handler(hook: &Value) -> bool {
    hook.get("url").and_then(Value::as_str) == Some(ENDPOINT)
        || hook
            .get("args")
            .and_then(Value::as_array)
            .is_some_and(|args| args.iter().any(|arg| arg.as_str() == Some("--crawde-hook")))
}

fn is_ours(group: &Value) -> bool {
    group
        .get("hooks")
        .and_then(Value::as_array)
        .is_some_and(|hooks| hooks.iter().any(is_our_handler))
}

pub fn merge_hooks(
    settings: &mut Value,
    token: &str,
    install: bool,
    executable: &str,
) -> Result<(), String> {
    let root = settings
        .as_object_mut()
        .ok_or("Claude settings must be a JSON object")?;
    let hooks = root
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or("Claude hooks must be an object")?;
    for event in EVENTS {
        let groups = hooks
            .entry(*event)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .ok_or("Hook event must be an array")?;
        // Remove only Crawde handlers, preserving unrelated hooks in the same group.
        for group in groups.iter_mut().filter(|g| is_ours(g)) {
            if let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                handlers.retain(|h| !is_our_handler(h));
            }
        }
        groups.retain(|g| {
            g.get("hooks")
                .and_then(Value::as_array)
                .is_none_or(|v| !v.is_empty())
        });
        if install {
            if *event == "SessionStart" {
                groups.push(json!({"hooks":[{"type":"command", "command":executable, "args":["--crawde-hook"], "timeout":2}]}));
            } else {
                groups.push(json!({"hooks":[{
                    "type":"http", "url":ENDPOINT, "timeout":1,
                    "headers":{"Authorization":format!("Bearer {token}")}
                }]}));
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn configure_hooks(app: AppHandle, install: bool) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("settings.json");
    let contents = if path.exists() {
        fs::read_to_string(&path).map_err(|e| e.to_string())?
    } else {
        "{}".into()
    };
    let mut settings: Value = serde_json::from_str(&contents)
        .map_err(|e| format!("Claude settings were left unchanged: {e}"))?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    merge_hooks(
        &mut settings,
        &token(&app)?,
        install,
        &executable.to_string_lossy(),
    )?;
    // Keep an exact backup before every change; never rewrite malformed config.
    if path.exists() {
        fs::copy(&path, dir.join(format!("settings.crawde-{}.bak", now())))
            .map_err(|e| e.to_string())?;
    }
    let staging = dir.join("settings.crawde.tmp");
    fs::write(
        &staging,
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(staging, path).map_err(|e| e.to_string())
}

pub fn installed() -> bool {
    fs::read_to_string(config_dir().join("settings.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.get("hooks").cloned())
        .and_then(|v| {
            v.as_object().map(|m| {
                m.values()
                    .any(|groups| groups.as_array().is_some_and(|g| g.iter().any(is_ours)))
            })
        })
        .unwrap_or(false)
}

fn startup_hook_matches(settings: &Value, executable: &str) -> bool {
    settings
        .get("hooks")
        .and_then(|h| h.get("SessionStart"))
        .and_then(Value::as_array)
        .is_some_and(|groups| {
            groups.iter().any(|group| {
                group
                    .get("hooks")
                    .and_then(Value::as_array)
                    .is_some_and(|hooks| {
                        hooks.iter().any(|hook| {
                            is_our_handler(hook)
                                && hook.get("type").and_then(Value::as_str) == Some("command")
                                && hook.get("command").and_then(Value::as_str) == Some(executable)
                        })
                    })
            })
        })
}

pub fn needs_repair() -> bool {
    let Ok(executable) = std::env::current_exe() else {
        return true;
    };
    fs::read_to_string(config_dir().join("settings.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .is_none_or(|settings| !startup_hook_matches(&settings, &executable.to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn install_is_idempotent_and_uninstall_preserves_other_hooks() {
        let mut settings = json!({"theme":"dark","hooks":{"Stop":[{"hooks":[{"type":"command","command":"echo hi"}]}]}});
        merge_hooks(&mut settings, "secret", true, "C:\\Crawde\\crawde.exe").unwrap();
        merge_hooks(&mut settings, "secret", true, "C:\\Crawde\\crawde.exe").unwrap();
        assert_eq!(
            settings["hooks"]["SessionStart"][0]["hooks"][0]["type"],
            "command"
        );
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 2);
        merge_hooks(&mut settings, "secret", false, "C:\\Crawde\\crawde.exe").unwrap();
        assert!(settings["hooks"]["SessionStart"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
        assert_eq!(settings["theme"], "dark");
    }

    #[test]
    fn repairs_legacy_http_startup_without_removing_other_handlers() {
        let mut settings = json!({"hooks":{"SessionStart":[{"hooks":[
            {"type":"http","url":ENDPOINT}, {"type":"command","command":"keep-me"}
        ]}]}});
        assert!(!startup_hook_matches(&settings, "C:\\Crawde\\crawde.exe"));
        merge_hooks(&mut settings, "secret", true, "C:\\Crawde\\crawde.exe").unwrap();
        assert!(startup_hook_matches(&settings, "C:\\Crawde\\crawde.exe"));
        assert_eq!(
            settings["hooks"]["SessionStart"][0]["hooks"][0]["command"],
            "keep-me"
        );
    }
}
