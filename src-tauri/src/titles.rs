use serde_json::Value;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

fn custom_title(line: &str, session_id: &str) -> Option<String> {
    let value: Value = serde_json::from_str(line).ok()?;
    if value.get("type")?.as_str()? != "custom-title"
        || value.get("sessionId")?.as_str()? != session_id
    {
        return None;
    }
    value
        .get("customTitle")?
        .as_str()
        .map(|s| crate::activity::short(s, 140))
}

fn first_prompt(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let value: Value = serde_json::from_str(line).ok()?;
        if value.get("type")?.as_str()? != "user"
            || value
                .get("isMeta")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        {
            return None;
        }
        let content = value.get("message")?.get("content")?;
        let text = content.as_str().or_else(|| {
            content.as_array()?.iter().find_map(|block| {
                if block.get("type")?.as_str()? == "text" {
                    block.get("text")?.as_str()
                } else {
                    None
                }
            })
        })?;
        // Claude's command/meta messages are not conversation names.
        if text.starts_with('<') {
            return None;
        }
        Some(crate::activity::short(text, 90))
    })
}

fn wsl_transcript(session_id: &str, transcript: &str, distro: &str) -> Option<std::path::PathBuf> {
    let segments: Vec<&str> = transcript.strip_prefix('/')?.split('/').collect();
    let valid = !distro.is_empty()
        && !distro.contains(['/', '\\'])
        && segments.last() == Some(&format!("{session_id}.jsonl").as_str())
        && segments.iter().all(|s| !s.is_empty() && *s != "..")
        && segments
            .windows(2)
            .any(|pair| pair == [".claude", "projects"]);
    valid.then(|| {
        std::path::PathBuf::from(format!(r"\\wsl.localhost\{distro}\{}", segments.join("\\")))
    })
}

pub fn transcript_file(
    session_id: &str,
    transcript: &str,
    distro: Option<&str>,
) -> Option<std::path::PathBuf> {
    if transcript.is_empty() {
        return None;
    }
    if let Some(distro) = distro {
        return wsl_transcript(session_id, transcript, distro);
    }
    let path = Path::new(transcript).canonicalize().ok()?;
    let root = crate::bridge::config_dir()
        .join("projects")
        .canonicalize()
        .ok()?;
    if !path.starts_with(root) || path.file_name()?.to_str()? != format!("{session_id}.jsonl") {
        return None;
    }
    Some(path)
}

pub fn read_title(session_id: &str, transcript: &str, distro: Option<&str>) -> Option<String> {
    let path = transcript_file(session_id, transcript, distro)?;
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let mut head = Vec::new();
    (&mut file).take(65_536).read_to_end(&mut head).ok()?;
    file.seek(SeekFrom::Start(length.saturating_sub(65_536)))
        .ok()?;
    let mut tail = Vec::new();
    file.take(65_536).read_to_end(&mut tail).ok()?;
    let tail = String::from_utf8_lossy(&tail);
    let head = String::from_utf8_lossy(&head);
    tail.lines()
        .rev()
        .find_map(|line| custom_title(line, session_id))
        .or_else(|| {
            head.lines()
                .rev()
                .find_map(|line| custom_title(line, session_id))
        })
        .or_else(|| first_prompt(&head))
        .filter(|title| !title.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wsl_transcripts_map_to_unc_paths_and_reject_traversal() {
        let ok = "/home/u/.claude/projects/-home-u-app/abc.jsonl";
        assert_eq!(
            wsl_transcript("abc", ok, "Ubuntu")
                .unwrap()
                .to_str()
                .unwrap(),
            r"\\wsl.localhost\Ubuntu\home\u\.claude\projects\-home-u-app\abc.jsonl"
        );
        assert!(wsl_transcript("abc", ok, "..\\x").is_none());
        assert!(wsl_transcript("other", ok, "Ubuntu").is_none());
        assert!(wsl_transcript("abc", "/home/u/.claude/projects/../abc.jsonl", "Ubuntu").is_none());
        assert!(wsl_transcript("abc", "/etc/abc.jsonl", "Ubuntu").is_none());
    }

    #[test]
    fn transcript_names_ignore_meta_and_accept_custom_titles() {
        let lines = "{\"type\":\"user\",\"isMeta\":true,\"message\":{\"content\":\"metadata\"}}\n{\"type\":\"user\",\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"Build a crab companion\"}]}}";
        assert_eq!(
            first_prompt(lines).as_deref(),
            Some("Build a crab companion")
        );
        assert_eq!(
            custom_title(
                r#"{"type":"custom-title","sessionId":"abc","customTitle":"My crab"}"#,
                "abc"
            )
            .as_deref(),
            Some("My crab")
        );
        assert!(custom_title(
            r#"{"type":"custom-title","sessionId":"other","customTitle":"Wrong"}"#,
            "abc"
        )
        .is_none());
    }
}
