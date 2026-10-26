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

pub fn read_title(session_id: &str, transcript: &str) -> Option<String> {
    if transcript.is_empty() {
        return None;
    }
    let path = Path::new(transcript).canonicalize().ok()?;
    let root = crate::bridge::config_dir()
        .join("projects")
        .canonicalize()
        .ok()?;
    if !path.starts_with(root) || path.file_name()?.to_str()? != format!("{session_id}.jsonl") {
        return None;
    }
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
