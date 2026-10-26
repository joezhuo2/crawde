use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

const RELEASES: &str = "https://github.com/joezhuo2/crawde/releases";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub checked_at: u64,
    pub available: bool,
    pub version: Option<String>,
    pub message: String,
}

#[derive(Deserialize)]
struct Release { tag_name: String }

#[tauri::command]
pub async fn check_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(12))
        .user_agent("Crawde/0.1.0").build().map_err(|e| e.to_string())?;
    let response = client.get("https://api.github.com/repos/joezhuo2/crawde/releases/latest")
        .send().await.map_err(|e| format!("Could not reach GitHub: {e}"))?;
    let info = if response.status() == reqwest::StatusCode::NOT_FOUND {
        UpdateInfo { checked_at: crate::bridge::now(), available: false, version: None, message: "No public release is available yet.".into() }
    } else {
        let release = response.error_for_status().map_err(|e| format!("GitHub update check failed: {e}"))?
            .json::<Release>().await.map_err(|e| e.to_string())?;
        let latest = semver::Version::parse(release.tag_name.trim_start_matches('v')).map_err(|e| e.to_string())?;
        let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
        let available = latest > current;
        UpdateInfo { checked_at: crate::bridge::now(), available, version: Some(latest.to_string()),
            message: if available { format!("Crawde {latest} is available. View releases to download it.") } else { "You're up to date.".into() } }
    };
    *app.state::<crate::AppState>().update.lock().map_err(|e| e.to_string())? = Some(info.clone());
    Ok(info)
}

pub fn check_on_open(app: &AppHandle) {
    let state = app.state::<crate::AppState>();
    if !state.preferences.lock().unwrap().check_on_open { return; }
    let recent = state.update.lock().unwrap().as_ref().is_some_and(|i| crate::bridge::now().saturating_sub(i.checked_at) < 1_800_000);
    if recent { return; }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = check_updates(app.clone()).await {
            *app.state::<crate::AppState>().update.lock().unwrap() = Some(UpdateInfo {
                checked_at: crate::bridge::now(), available: false, version: None, message: error,
            });
        }
    });
}

#[tauri::command]
pub fn open_releases(app: AppHandle) -> Result<(), String> {
    app.opener().open_url(RELEASES, None::<&str>).map_err(|e| e.to_string())
}
