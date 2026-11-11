use crate::AppState;
use std::{path::PathBuf, process::Command};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Default)]
pub struct Navigation {
    pub current_project: Option<PathBuf>,
    pub current_distro: Option<String>,
}

#[tauri::command]
pub fn open_claude(app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_url("claude://", None::<&str>)
        .map_err(|e| format!("Could not open Claude. Check that Claude Desktop is installed: {e}"))
}

fn valid_wsl(distro: &str, cwd: &str) -> Result<(), String> {
    if !crate::wsl::valid_distro(distro) || !cwd.starts_with('/') || cwd.contains('\0') {
        return Err("Invalid WSL project".into());
    }
    Ok(())
}

fn wsl_folder(distro: &str, cwd: &str) -> PathBuf {
    PathBuf::from(format!(
        r"\\wsl.localhost\{distro}{}",
        cwd.replace('/', "\\")
    ))
}

fn open_wsl(
    app: &AppHandle,
    target: &str,
    distro: &str,
    cwd: &str,
    id: Option<&str>,
) -> Result<(), String> {
    valid_wsl(distro, cwd)?;
    if target == "terminal" {
        return launch_wsl(distro, cwd, id);
    }
    app.opener()
        .open_url(new_code_url(Some(&wsl_folder(distro, cwd))), None::<&str>)
        .map_err(|e| e.to_string())
}

fn launch_wsl(distro: &str, cwd: &str, id: Option<&str>) -> Result<(), String> {
    valid_wsl(distro, cwd)?;
    let mut args = vec![
        "--distribution",
        distro,
        "--cd",
        cwd,
        "--exec",
        "bash",
        "-lic",
        "claude \"$@\" || exec bash -li",
        "crawde",
    ];
    if let Some(id) = id {
        args.extend(["--resume", id]);
    }
    let mut terminal = Command::new("wt.exe");
    terminal.args(["new-tab", "wsl.exe"]).args(&args);
    if terminal.spawn().is_ok() {
        return Ok(());
    }
    let mut terminal = Command::new("wsl.exe");
    terminal.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        terminal.creation_flags(0x00000010);
    }
    terminal.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(windows)]
fn claude_exe() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_default();
    let native = home.join(".local/bin/claude.exe");
    if native.is_file() {
        return Ok(native);
    }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let path = dir.join("claude.exe");
        if path.is_file() {
            return Ok(path);
        }
    }
    Err("Install the native Claude Code CLI to open conversations: https://code.claude.com/docs/en/setup".into())
}

#[cfg(not(windows))]
fn claude_exe() -> Result<PathBuf, String> {
    let home = crate::platform::home_dir();
    let mut candidates = vec![
        home.join(".local/bin/claude"),
        home.join(".claude/local/claude"),
        home.join(".npm-global/bin/claude"),
        home.join(".bun/bin/claude"),
        PathBuf::from("/opt/homebrew/bin/claude"),
        PathBuf::from("/usr/local/bin/claude"),
    ];
    candidates.extend(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|dir| dir.join("claude")),
    );
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "Install the native Claude Code CLI to open conversations: https://code.claude.com/docs/en/setup".into())
}

#[cfg(windows)]
const TERMINAL_NAME: &str = "Windows Terminal";
#[cfg(not(windows))]
const TERMINAL_NAME: &str = "Terminal";

fn folder(cwd: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(cwd);
    if !path.is_absolute() || !path.is_dir() {
        return Err("This project folder is no longer available".into());
    }
    Ok(path)
}

#[cfg_attr(windows, allow(dead_code))]
fn terminal_script(project: &str, executable: &str, id: Option<&str>) -> String {
    use crate::platform::shell_quote;
    let mut command = format!("exec {}", shell_quote(executable));
    if let Some(id) = id {
        command.push_str(&format!(" --resume {}", shell_quote(id)));
    }
    format!(
        "#!/bin/sh\nrm -f -- \"$0\"\ncd {} || exit 1\n{command}\n",
        shell_quote(project)
    )
}

#[cfg(not(windows))]
fn launch_terminal(project: &PathBuf, id: Option<&str>) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let executable = claude_exe()?;
    let script = std::env::temp_dir().join(format!("crawde-{}.command", uuid::Uuid::new_v4()));
    std::fs::write(
        &script,
        terminal_script(
            &project.to_string_lossy(),
            &executable.to_string_lossy(),
            id,
        ),
    )
    .map_err(|e| e.to_string())?;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| e.to_string())?;
    let opened = Command::new("open")
        .args(["-a", "Terminal"])
        .arg(&script)
        .status()
        .is_ok_and(|status| status.success());
    if !opened {
        let _ = std::fs::remove_file(&script);
        return Err("Could not open Terminal".into());
    }
    Ok(())
}

#[cfg(windows)]
fn launch_terminal(project: &PathBuf, id: Option<&str>) -> Result<(), String> {
    let executable = claude_exe()?;
    let mut terminal = Command::new("wt.exe");
    terminal
        .args(["new-tab", "--startingDirectory"])
        .arg(project)
        .arg(&executable);
    if let Some(id) = id {
        terminal.args(["--resume", id]);
    }
    // Argument arrays preserve spaces and special characters without shell interpolation.
    if terminal.spawn().is_ok() {
        return Ok(());
    }
    let mut process = Command::new(executable);
    process.current_dir(project);
    if let Some(id) = id {
        process.args(["--resume", id]);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x00000010); // CREATE_NEW_CONSOLE: user-requested interactive Claude session.
    }
    process.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

// Claude's Desktop handoff requires a console: CREATE_NO_WINDOW or piped
// stdout makes the CLI reject --desktop as non-interactive. Start-Process gives
// it a real, hidden console. Values travel through environment variables rather
// than being interpolated into PowerShell source.
fn launch_desktop(executable: &PathBuf, project: &PathBuf, id: &str) -> Result<(), String> {
    #[cfg(windows)]
    let mut process = {
        use std::os::windows::process::CommandExt;
        let mut command = Command::new("powershell.exe");
        command
            .args(["-NoProfile", "-NonInteractive", "-Command",
                "$ErrorActionPreference = 'Stop'; $p = Start-Process -FilePath $env:CRAWDE_CLI -WorkingDirectory $env:CRAWDE_PROJECT -ArgumentList @('--desktop', '--resume', $env:CRAWDE_SESSION) -WindowStyle Hidden -PassThru; $handle = $p.Handle; if (-not $p.WaitForExit(12000)) { $p.Kill(); exit 1 }; exit $p.ExitCode"])
            .env("CRAWDE_CLI", executable)
            .env("CRAWDE_PROJECT", project)
            .env("CRAWDE_SESSION", id)
            .creation_flags(0x08000000); // Hide the launcher; the CLI gets its own console.
        command
    };
    #[cfg(not(windows))]
    let mut process = {
        #[cfg(target_os = "macos")]
        let mut command = {
            let mut command = Command::new("script");
            command.args(["-q", "/dev/null"]).arg(executable);
            command
        };
        #[cfg(not(target_os = "macos"))]
        let mut command = Command::new(executable);
        command
            .current_dir(project)
            .args(["--desktop", "--resume", id])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        command
    };
    #[cfg(windows)]
    let status = process.status();
    #[cfg(not(windows))]
    let status = status_within(&mut process, std::time::Duration::from_secs(12));
    let status = status.map_err(|e| format!("Could not open this conversation: {e}"))?;
    if !status.success() {
        return Err(format!("Claude could not open this conversation in Desktop. Check that Claude Desktop is installed and the conversation is still available, or select {TERMINAL_NAME} in Settings → Claude."));
    }
    Ok(())
}

#[cfg(not(windows))]
fn status_within(
    process: &mut Command,
    limit: std::time::Duration,
) -> std::io::Result<std::process::ExitStatus> {
    let mut child = process.spawn()?;
    let deadline = std::time::Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Claude did not finish within 12 seconds",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

#[tauri::command]
pub async fn open_chat(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let session = state
        .activity
        .lock()
        .map_err(|e| e.to_string())?
        .sessions
        .get(&id)
        .cloned()
        .ok_or("Conversation has expired")?;
    let target = state
        .preferences
        .lock()
        .map_err(|e| e.to_string())?
        .open_target
        .clone();
    if let Some(distro) = &session.wsl_distro {
        open_wsl(&app, &target, distro, &session.cwd, Some(&id))?;
        let mut navigation = state.navigation.lock().map_err(|e| e.to_string())?;
        navigation.current_project = Some(PathBuf::from(session.cwd));
        navigation.current_distro = Some(distro.clone());
        return Ok(());
    }
    let project = folder(&session.cwd)?;
    if target == "terminal" {
        launch_terminal(&project, Some(&id))?;
    } else {
        let executable = claude_exe()?;
        let desktop_project = project.clone();
        let desktop_id = id.clone();
        let handoff = tauri::async_runtime::spawn_blocking(move || {
            launch_desktop(&executable, &desktop_project, &desktop_id)
        })
        .await
        .map_err(|e| e.to_string())?;
        if let Err(error) = handoff {
            app.opener()
                .open_url(resume_url(&id), None::<&str>)
                .map_err(|_| error)?;
        }
    }
    let mut navigation = state.navigation.lock().map_err(|e| e.to_string())?;
    navigation.current_project = Some(project);
    navigation.current_distro = None;
    Ok(())
}

fn resume_url(id: &str) -> String {
    let mut url = url::Url::parse("claude://resume").expect("constant URL");
    url.query_pairs_mut().append_pair("session", id);
    url.into()
}

pub fn new_code_url(project: Option<&PathBuf>) -> String {
    let mut url = url::Url::parse("claude://code/new").expect("constant URL");
    if let Some(path) = project {
        url.query_pairs_mut()
            .append_pair("folder", &path.to_string_lossy());
    }
    url.into()
}

#[tauri::command]
pub fn new_chat(app: AppHandle, state: State<AppState>, id: Option<String>) -> Result<(), String> {
    let target = state
        .preferences
        .lock()
        .map_err(|e| e.to_string())?
        .open_target
        .clone();
    {
        let navigation = state.navigation.lock().map_err(|e| e.to_string())?;
        if let (Some(distro), Some(project)) =
            (&navigation.current_distro, &navigation.current_project)
        {
            return open_wsl(&app, &target, distro, &project.to_string_lossy(), None);
        }
        if navigation.current_project.is_none() {
            let activity = state.activity.lock().map_err(|e| e.to_string())?;
            if let Some(session) = id
                .as_ref()
                .and_then(|id| activity.sessions.get(id))
                .or_else(|| activity.sessions.values().max_by_key(|s| s.updated_at))
            {
                if let Some(distro) = &session.wsl_distro {
                    return open_wsl(&app, &target, distro, &session.cwd, None);
                }
            }
        }
    }
    let selected = state
        .navigation
        .lock()
        .map_err(|e| e.to_string())?
        .current_project
        .clone();
    let project = if selected.is_some() {
        selected
    } else {
        let activity = state.activity.lock().map_err(|e| e.to_string())?;
        let session = id
            .as_ref()
            .and_then(|id| activity.sessions.get(id))
            .or_else(|| activity.sessions.values().max_by_key(|s| s.updated_at));
        session.map(|s| folder(&s.cwd)).transpose()?
    };
    if target == "terminal" {
        let project =
            project.ok_or("Open a Claude project first so Crawde knows where to start the chat")?;
        launch_terminal(&project, None)
    } else {
        app.opener()
            .open_url(new_code_url(project.as_ref()), None::<&str>)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn desktop_handoff_has_a_console_and_preserves_the_selected_chat() {
        let root = std::env::temp_dir().join(format!(
            "crawde navigation & spaces {}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let probe = root.join("probe.cmd");
        std::fs::write(&probe, concat!(
            "@echo off\r\n",
            "echo %* > args.txt\r\n",
            r#"powershell.exe -NoProfile -Command "[Console]::IsInputRedirected, [Console]::IsOutputRedirected | Set-Content console.txt""#, "\r\n",
            "exit /b 0\r\n"
        )).unwrap();
        launch_desktop(&probe, &root, "selected-chat-123").unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("args.txt"))
                .unwrap()
                .trim(),
            "--desktop --resume selected-chat-123"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("console.txt"))
                .unwrap()
                .trim(),
            "False\nFalse".replace('\n', "\r\n")
        );

        std::fs::write(&probe, "@exit /b 1\r\n").unwrap();
        assert!(launch_desktop(&probe, &root, "other-chat-456")
            .unwrap_err()
            .contains("could not open this conversation"));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn terminal_script_quotes_paths_and_removes_itself() {
        let script = terminal_script(
            "/Users/me/Crab & 'Co'",
            "/Users/me/.local/bin/claude",
            Some("abc-123"),
        );
        assert_eq!(
            script,
            "#!/bin/sh\nrm -f -- \"$0\"\ncd '/Users/me/Crab & '\\''Co'\\''' || exit 1\nexec '/Users/me/.local/bin/claude' --resume 'abc-123'\n"
        );
        assert!(!terminal_script("/tmp", "/bin/claude", None).contains("--resume"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn desktop_handoff_has_a_terminal_and_preserves_the_selected_chat() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!(
            "crawde navigation & spaces {}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let probe = root.join("probe.sh");
        std::fs::write(
            &probe,
            "#!/bin/sh\necho \"$@\" > args.txt\nif [ -t 0 ] && [ -t 1 ]; then echo tty > tty.txt; fi\n",
        )
        .unwrap();
        std::fs::set_permissions(&probe, std::fs::Permissions::from_mode(0o755)).unwrap();
        launch_desktop(&probe, &root, "selected-chat-123").unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("args.txt"))
                .unwrap()
                .trim(),
            "--desktop --resume selected-chat-123"
        );
        assert!(root.join("tty.txt").is_file());

        std::fs::write(&probe, "#!/bin/sh\nexit 1\n").unwrap();
        assert!(launch_desktop(&probe, &root, "other-chat-456")
            .unwrap_err()
            .contains("could not open this conversation"));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn project_link_encodes_spaces_and_special_characters() {
        let path = PathBuf::from("C:\\Projects\\Crab & Co");
        let url = url::Url::parse(&new_code_url(Some(&path))).unwrap();
        assert_eq!(
            url.query_pairs().find(|(k, _)| k == "folder").unwrap().1,
            path.to_string_lossy()
        );
        assert_eq!(new_code_url(None), "claude://code/new");
    }

    #[test]
    fn resume_link_targets_the_selected_session() {
        assert_eq!(
            resume_url("ae91279c-8e58-46ae-a9da-5fb294c4276f"),
            "claude://resume?session=ae91279c-8e58-46ae-a9da-5fb294c4276f"
        );
    }
}
