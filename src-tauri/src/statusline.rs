use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

fn percent_in(directory: &Path, session_id: &str) -> Option<u8> {
    if !crate::activity::valid_session_id(session_id) {
        return None;
    }
    let mut text = String::new();
    File::open(directory.join(format!("{session_id}.json")))
        .ok()?
        .take(65_537)
        .read_to_string(&mut text)
        .ok()?;
    if text.len() > 65_536 {
        return None;
    }
    let value: Value = serde_json::from_str(&text).ok()?;
    let used = value
        .get("context_window")?
        .get("used_percentage")?
        .as_f64()?;
    Some(used.clamp(0.0, 100.0).round() as u8)
}

pub fn context_pct(session_id: &str) -> Option<u8> {
    percent_in(
        &crate::platform::home_dir().join(".crawde/statusline"),
        session_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch() -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("crawde-statusline-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    fn pct(body: &str) -> Option<u8> {
        let directory = scratch();
        fs::write(directory.join("abc-1.json"), body).unwrap();
        let result = percent_in(&directory, "abc-1");
        fs::remove_dir_all(directory).unwrap();
        result
    }

    #[test]
    fn rounds_and_clamps_used_percentage() {
        assert_eq!(
            pct(r#"{"context_window":{"used_percentage":41.4}}"#),
            Some(41)
        );
        assert_eq!(
            pct(r#"{"context_window":{"used_percentage":41.5}}"#),
            Some(42)
        );
        assert_eq!(pct(r#"{"context_window":{"used_percentage":0}}"#), Some(0));
        assert_eq!(
            pct(r#"{"context_window":{"used_percentage":250}}"#),
            Some(100)
        );
        assert_eq!(pct(r#"{"context_window":{"used_percentage":-3}}"#), Some(0));
    }

    #[test]
    fn missing_null_or_malformed_data_yields_none() {
        assert_eq!(pct(r#"{"context_window":{"used_percentage":null}}"#), None);
        assert_eq!(pct(r#"{"context_window":{}}"#), None);
        assert_eq!(pct(r#"{"context_window":{"used_percentage":"41"}}"#), None);
        assert_eq!(pct(r#"{"session_id":"abc-1"}"#), None);
        assert_eq!(pct("not json"), None);
        assert_eq!(pct(""), None);
    }

    #[test]
    fn absent_file_and_invalid_session_ids_yield_none() {
        let directory = scratch();
        fs::write(
            directory.join("abc-1.json"),
            r#"{"context_window":{"used_percentage":10}}"#,
        )
        .unwrap();
        assert_eq!(percent_in(&directory, "abc-1"), Some(10));
        assert_eq!(percent_in(&directory, "other"), None);
        assert_eq!(percent_in(&directory, ""), None);
        assert_eq!(percent_in(&directory, "../abc-1"), None);
        assert_eq!(percent_in(&directory, "abc/../abc-1"), None);
        assert_eq!(percent_in(&directory, &"a".repeat(129)), None);
        fs::remove_dir_all(directory).unwrap();
    }
}
