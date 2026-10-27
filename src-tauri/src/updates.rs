use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

const RELEASES: &str = "https://github.com/joezhuo2/crawde/releases";
const API: &str = "https://api.github.com/repos/joezhuo2/crawde/releases/latest";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub checked_at: u64,
    pub available: bool,
    pub check_succeeded: bool,
    pub version: Option<String>,
    pub message: String,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    assets: Vec<Asset>,
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .user_agent(format!("Crawde/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())
}

async fn latest(client: &reqwest::Client) -> Result<Option<Release>, String> {
    let response = client
        .get(API)
        .timeout(std::time::Duration::from_secs(12))
        .send()
        .await
        .map_err(|e| format!("Could not reach GitHub: {e}"))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    response
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map(Some)
        .map_err(|e| e.to_string())
}

fn newer(release: &Release) -> Result<Option<semver::Version>, String> {
    let latest = semver::Version::parse(release.tag_name.trim_start_matches('v'))
        .map_err(|e| e.to_string())?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    Ok((latest > current).then_some(latest))
}

fn status(
    app: &AppHandle,
    message: String,
    version: Option<String>,
    check_succeeded: bool,
) -> UpdateInfo {
    let info = UpdateInfo {
        checked_at: crate::bridge::now(),
        available: version.is_some(),
        check_succeeded,
        version,
        message,
    };
    *app.state::<crate::AppState>().update.lock().unwrap() = Some(info.clone());
    crate::lifecycle::refresh_menu(app);
    info
}

#[tauri::command]
pub async fn check_updates(app: AppHandle) -> Result<UpdateInfo, String> {
    let release = latest(&client()?).await?;
    let version = release
        .as_ref()
        .map(newer)
        .transpose()?
        .flatten()
        .map(|v| v.to_string());
    let message = match (&release, &version) {
        (None, _) => "No public release is available yet.".into(),
        (_, Some(v)) => format!("Crawde {v} is available."),
        _ => "You're up to date.".into(),
    };
    Ok(status(&app, message, version, true))
}

fn asset_url<'a>(release: &'a Release, name: &str) -> Result<&'a str, String> {
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| format!("Release is missing {name}"))?;
    let prefix = format!("{RELEASES}/download/{}/", release.tag_name);
    if !asset.browser_download_url.starts_with(&prefix) {
        return Err("Unexpected release download URL".into());
    }
    Ok(&asset.browser_download_url)
}

fn verify(bytes: &[u8], checksum: &str, filename: &str) -> Result<(), String> {
    let fields: Vec<_> = checksum.split_whitespace().collect();
    let hash = format!("{:x}", Sha256::digest(bytes));
    if fields.len() != 2
        || !fields[0].eq_ignore_ascii_case(&hash)
        || fields[1].trim_start_matches('*') != filename
    {
        return Err("Installer checksum did not match. Crawde has not closed.".into());
    }
    Ok(())
}

async fn install(app: &AppHandle) -> Result<(), String> {
    let client = client()?;
    let release = latest(&client)
        .await?
        .ok_or("No public release is available yet.")?;
    let Some(version) = newer(&release)? else {
        status(app, "You're up to date.".into(), None, true);
        return Ok(());
    };
    let filename = format!("Crawde_{version}_x64-setup.exe");
    status(
        app,
        format!("Downloading Crawde {version}…"),
        Some(version.to_string()),
        true,
    );
    let checksum = client
        .get(asset_url(&release, &format!("{filename}.sha256"))?)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let bytes = client
        .get(asset_url(&release, &filename)?)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;
    verify(&bytes, &checksum, &filename)?;
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("updates")
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(filename);
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    std::process::Command::new(&path)
        .spawn()
        .map_err(|e| format!("Could not launch installer: {e}"))?;
    app.exit(0);
    Ok(())
}

pub fn start(app: &AppHandle, install_latest: bool) {
    if app
        .state::<crate::AppState>()
        .update_busy
        .swap(true, Ordering::SeqCst)
    {
        return;
    }
    status(app, "Checking for updates…".into(), None, false);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = if install_latest {
            install(&app).await
        } else {
            check_updates(app.clone()).await.map(|_| ())
        };
        if let Err(error) = result {
            status(&app, error, None, false);
        }
        app.state::<crate::AppState>()
            .update_busy
            .store(false, Ordering::SeqCst);
        crate::lifecycle::refresh_menu(&app);
    });
}

pub fn check_on_launch(app: &AppHandle) {
    start(app, false);
}

pub fn menu_label(info: Option<&UpdateInfo>, busy: bool) -> (String, bool) {
    if busy {
        return (
            info.map(|i| i.message.clone())
                .unwrap_or_else(|| "Checking for updates…".into()),
            false,
        );
    }
    match info {
        Some(info) if info.check_succeeded && info.available => {
            if let Some(version) = &info.version {
                return (format!("Install latest update (v{version})"), true);
            }
            ("Update check failed — retry".into(), true)
        }
        Some(info) if info.check_succeeded => {
            (format!("Up to date (v{})", env!("CARGO_PKG_VERSION")), true)
        }
        Some(_) => ("Update check failed — retry".into(), true),
        None => ("Checking for updates…".into(), false),
    }
}

pub fn activate(app: &AppHandle) {
    let install_latest = app
        .state::<crate::AppState>()
        .update
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|info| info.check_succeeded && info.available && info.version.is_some());
    start(app, install_latest);
}

#[tauri::command]
pub fn open_releases(app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(RELEASES, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn update_menu_tracks_version_progress_and_failures() {
        let mut info = UpdateInfo {
            checked_at: 1,
            available: false,
            check_succeeded: true,
            version: None,
            message: "You're up to date.".into(),
        };
        assert_eq!(
            menu_label(Some(&info), false),
            (format!("Up to date (v{})", env!("CARGO_PKG_VERSION")), true)
        );
        info.available = true;
        info.version = Some("9.0.0".into());
        assert_eq!(
            menu_label(Some(&info), false),
            ("Install latest update (v9.0.0)".into(), true)
        );
        info.message = "Downloading Crawde 9.0.0…".into();
        assert_eq!(menu_label(Some(&info), true), (info.message.clone(), false));
        info.check_succeeded = false;
        assert_eq!(
            menu_label(Some(&info), false),
            ("Update check failed — retry".into(), true)
        );
        assert_eq!(
            menu_label(None, false),
            ("Checking for updates…".into(), false)
        );
    }

    #[test]
    fn checksum_requires_matching_content_and_filename() {
        let checksum = format!("{:x}  setup.exe\n", Sha256::digest(b"installer"));
        assert!(verify(b"installer", &checksum, "setup.exe").is_ok());
        assert!(verify(b"corrupted", &checksum, "setup.exe").is_err());
        assert!(verify(b"installer", &checksum, "other.exe").is_err());
        assert!(verify(b"installer", "", "setup.exe").is_err());
    }
    #[test]
    fn download_requires_exact_release_asset() {
        let release = Release {
            tag_name: "v1.0.0".into(),
            assets: vec![Asset {
                name: "setup.exe".into(),
                browser_download_url: format!("{RELEASES}/download/v1.0.0/setup.exe"),
            }],
        };
        assert!(asset_url(&release, "setup.exe").is_ok());
        assert!(asset_url(&release, "missing.exe").is_err());
        let bad = Release {
            tag_name: "v1.0.0".into(),
            assets: vec![Asset {
                name: "setup.exe".into(),
                browser_download_url: "https://example.com/setup.exe".into(),
            }],
        };
        assert!(asset_url(&bad, "setup.exe").is_err());
    }
}
