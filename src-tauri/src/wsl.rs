use std::{
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub fn quiet_command() -> Command {
    let mut command = Command::new("wsl.exe");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}

fn output(mut command: Command, input: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .ok_or("Missing WSL stdin")?
            .write_all(input)
            .map_err(|e| e.to_string())?;
    } else {
        drop(child.stdin.take());
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if child.try_wait().map_err(|e| e.to_string())?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("WSL did not respond within 15 seconds".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(
            "WSL hook setup failed; check Python 3 and Windows interop in the distro".into(),
        );
    }
    Ok(result.stdout)
}

pub fn configure(executable: &str, install: bool, events: &[&str]) -> Result<(), String> {
    let mut list = quiet_command();
    list.args(["--list", "--quiet"]);
    let bytes = match output(list, None) {
        Ok(bytes) => bytes,
        // WSL is optional on Windows machines.
        Err(_) => return Ok(()),
    };
    let names = String::from_utf16_lossy(
        &bytes
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect::<Vec<_>>(),
    );
    let mut failures = Vec::new();
    for distro in names
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("docker-desktop"))
    {
        let mut command = quiet_command();
        command.args([
            "--distribution",
            distro,
            "--exec",
            "python3",
            "-c",
            include_str!("wsl_setup.py"),
        ]);
        let payload = serde_json::to_vec(
            &serde_json::json!({"executable":executable,"install":install,"events":events}),
        )
        .map_err(|e| e.to_string())?;
        if let Err(error) = output(command, Some(&payload)) {
            failures.push(format!("{distro}: {error}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Windows hooks updated, but {}",
            failures.join("; ")
        ))
    }
}

pub fn valid_distro(distro: &str) -> bool {
    !distro.is_empty()
        && distro.len() <= 128
        && distro
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distro_names_cannot_escape_a_unc_root() {
        assert!(valid_distro("Ubuntu-24.04"));
        for name in [
            "",
            "../Ubuntu",
            "Ubuntu\\share",
            "Ubuntu/other",
            "--exec sh",
        ] {
            assert!(!valid_distro(name));
        }
    }
}
