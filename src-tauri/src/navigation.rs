use std::{path::PathBuf, process::Command};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use crate::AppState;

#[derive(Default)]
pub struct Navigation { pub current_project: Option<PathBuf> }

fn claude_exe() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_default();
    let native = home.join(".local/bin/claude.exe");
    if native.is_file() { return Ok(native); }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let path = dir.join("claude.exe");
        if path.is_file() { return Ok(path); }
    }
    Err("Install the native Claude Code CLI to open conversations: https://code.claude.com/docs/en/setup".into())
}

fn folder(cwd: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(cwd);
    if !path.is_absolute() || !path.is_dir() { return Err("This project folder is no longer available".into()); }
    Ok(path)
}

fn launch_terminal(project: &PathBuf, id: Option<&str>) -> Result<(), String> {
    let executable = claude_exe()?;
    let mut terminal = Command::new("wt.exe");
    terminal.args(["new-tab", "--startingDirectory"]).arg(project).arg(&executable);
    if let Some(id) = id { terminal.args(["--resume", id]); }
    // Argument arrays preserve spaces and special characters without shell interpolation.
    if terminal.spawn().is_ok() { return Ok(()); }
    let mut process = Command::new(executable);
    process.current_dir(project);
    if let Some(id) = id { process.args(["--resume", id]); }
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x00000010); // CREATE_NEW_CONSOLE: user-requested interactive Claude session.
    }
    process.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_chat(app: AppHandle, state: State<AppState>, id: String) -> Result<(), String> {
    let session = state.activity.lock().map_err(|e| e.to_string())?.sessions.get(&id).cloned().ok_or("Conversation has expired")?;
    let project = folder(&session.cwd)?;
    let target = state.preferences.lock().map_err(|e| e.to_string())?.open_target.clone();
    if target == "terminal" { launch_terminal(&project, Some(&id))?; }
    else {
        let mut process = Command::new(claude_exe()?);
        process.current_dir(&project).args(["--desktop", "--resume", &id]);
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            process.creation_flags(0x08000000); // CREATE_NO_WINDOW: handoff opens Claude Desktop itself.
        }
        process.spawn().map_err(|e| e.to_string())?;
    }
    state.navigation.lock().map_err(|e| e.to_string())?.current_project = Some(project);
    let _ = app;
    Ok(())
}

pub fn new_code_url(project: Option<&PathBuf>) -> String {
    let mut url = url::Url::parse("claude://code/new").expect("constant URL");
    if let Some(path) = project { url.query_pairs_mut().append_pair("folder", &path.to_string_lossy()); }
    url.into()
}

#[tauri::command]
pub fn new_chat(app: AppHandle, state: State<AppState>, id: Option<String>) -> Result<(), String> {
    let selected = state.navigation.lock().map_err(|e| e.to_string())?.current_project.clone();
    let project = if selected.is_some() { selected } else {
        let activity = state.activity.lock().map_err(|e| e.to_string())?;
        let session = id.as_ref().and_then(|id| activity.sessions.get(id))
            .or_else(|| activity.sessions.values().max_by_key(|s| s.updated_at));
        session.map(|s| folder(&s.cwd)).transpose()?
    };
    let target = state.preferences.lock().map_err(|e| e.to_string())?.open_target.clone();
    if target == "terminal" {
        let project = project.ok_or("Open a Claude project first so Crawde knows where to start the chat")?;
        launch_terminal(&project, None)
    } else { app.opener().open_url(new_code_url(project.as_ref()), None::<&str>).map_err(|e| e.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn project_link_encodes_spaces_and_special_characters() {
        let path = PathBuf::from("C:\\Projects\\Crab & Co");
        let url = url::Url::parse(&new_code_url(Some(&path))).unwrap();
        assert_eq!(url.query_pairs().find(|(k, _)| k == "folder").unwrap().1, path.to_string_lossy());
        assert_eq!(new_code_url(None), "claude://code/new");
    }
}
