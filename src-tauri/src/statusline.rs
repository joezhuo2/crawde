use serde_json::Value;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub pct: u8,
    pub resets_at: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub five_hour: Option<Window>,
    pub seven_day: Option<Window>,
}

fn read_json(path: &Path) -> Option<Value> {
    let mut text = String::new();
    File::open(path)
        .ok()?
        .take(65_537)
        .read_to_string(&mut text)
        .ok()?;
    if text.len() > 65_536 {
        return None;
    }
    serde_json::from_str(&text).ok()
}

fn percent(used: f64) -> u8 {
    used.clamp(0.0, 100.0).round() as u8
}

fn percent_in(directory: &Path, session_id: &str) -> Option<u8> {
    if !crate::activity::valid_session_id(session_id) {
        return None;
    }
    let value = read_json(&directory.join(format!("{session_id}.json")))?;
    let used = value
        .get("context_window")?
        .get("used_percentage")?
        .as_f64()?;
    Some(percent(used))
}

fn window(limits: &Value, key: &str) -> Option<Window> {
    let window = limits.get(key)?;
    let used = window.get("used_percentage")?.as_f64()?;
    let resets_at = window
        .get("resets_at")
        .and_then(Value::as_f64)
        .filter(|seconds| *seconds >= 0.0)
        .map(|seconds| seconds as u64);
    Some(Window {
        pct: percent(used),
        resets_at,
    })
}

fn usage_in(directory: &Path) -> Option<Usage> {
    let value = read_json(&directory.join("latest.json"))?;
    let limits = value.get("rate_limits")?;
    let five_hour = window(limits, "five_hour");
    let seven_day = window(limits, "seven_day");
    if five_hour.is_none() && seven_day.is_none() {
        return None;
    }
    Some(Usage {
        five_hour,
        seven_day,
    })
}

fn tokens_in(line: &str) -> Option<u64> {
    let value: Value = serde_json::from_str(line).ok()?;
    if value.get("type")?.as_str()? != "assistant"
        || value.get("isSidechain").and_then(Value::as_bool) == Some(true)
    {
        return None;
    }
    let usage = value.get("message")?.get("usage")?;
    let sum: u64 = [
        "input_tokens",
        "cache_creation_input_tokens",
        "cache_read_input_tokens",
    ]
    .iter()
    .filter_map(|key| usage.get(*key).and_then(Value::as_u64))
    .sum();
    (sum > 0).then_some(sum)
}

fn transcript_percent(path: &Path) -> Option<u8> {
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(262_144)))
        .ok()?;
    let mut tail = Vec::new();
    file.take(262_144).read_to_end(&mut tail).ok()?;
    let tail = String::from_utf8_lossy(&tail);
    let tokens = tail.lines().rev().find_map(tokens_in)?;
    let window = if tokens > 200_000 { 1_000_000.0 } else { 200_000.0 };
    Some(percent(tokens as f64 * 100.0 / window))
}

pub fn context_pct(session_id: &str, transcript: &str) -> Option<u8> {
    percent_in(
        &crate::platform::home_dir().join(".crawde/statusline"),
        session_id,
    )
    .or_else(|| {
        transcript_percent(&crate::titles::transcript_file(session_id, transcript)?)
    })
}

pub fn usage() -> Option<Usage> {
    usage_in(&crate::platform::home_dir().join(".crawde/statusline"))
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

    #[test]
    fn transcript_tokens_use_last_main_assistant_message() {
        let lines = [
            r#"{"type":"assistant","isSidechain":false,"message":{"usage":{"input_tokens":2,"cache_creation_input_tokens":3000,"cache_read_input_tokens":97000}}}"#,
            r#"{"type":"assistant","isSidechain":true,"message":{"usage":{"input_tokens":999999}}}"#,
            r#"{"type":"user","message":{"content":"hi"}}"#,
        ]
        .join("\n");
        assert_eq!(lines.lines().rev().find_map(tokens_in), Some(100_002));
        let directory = scratch();
        let file = directory.join("s.jsonl");
        fs::write(&file, lines).unwrap();
        assert_eq!(transcript_percent(&file), Some(50));
        fs::remove_dir_all(directory).unwrap();
    }

    fn usage(body: &str) -> Option<Usage> {
        let directory = scratch();
        fs::write(directory.join("latest.json"), body).unwrap();
        let result = usage_in(&directory);
        fs::remove_dir_all(directory).unwrap();
        result
    }

    fn window(pct: u8, resets_at: Option<u64>) -> Option<Window> {
        Some(Window { pct, resets_at })
    }

    #[test]
    fn reads_both_rate_limit_windows() {
        let result = usage(
            r#"{"rate_limits":{"five_hour":{"used_percentage":37.4,"resets_at":1700000000},"seven_day":{"used_percentage":62.5,"resets_at":1700500000}}}"#,
        )
        .unwrap();
        assert_eq!(result.five_hour, window(37, Some(1_700_000_000)));
        assert_eq!(result.seven_day, window(63, Some(1_700_500_000)));
    }

    #[test]
    fn clamps_usage_and_tolerates_missing_reset_times() {
        let result = usage(
            r#"{"rate_limits":{"five_hour":{"used_percentage":250},"seven_day":{"used_percentage":-4,"resets_at":null}}}"#,
        )
        .unwrap();
        assert_eq!(result.five_hour, window(100, None));
        assert_eq!(result.seven_day, window(0, None));
    }

    #[test]
    fn a_single_window_is_enough() {
        let only_five =
            usage(r#"{"rate_limits":{"five_hour":{"used_percentage":10,"resets_at":5}}}"#).unwrap();
        assert_eq!(only_five.five_hour, window(10, Some(5)));
        assert_eq!(only_five.seven_day, None);
        let only_week = usage(
            r#"{"rate_limits":{"five_hour":{"resets_at":5},"seven_day":{"used_percentage":90}}}"#,
        )
        .unwrap();
        assert_eq!(only_week.five_hour, None);
        assert_eq!(only_week.seven_day, window(90, None));
    }

    #[test]
    fn missing_or_malformed_rate_limits_yield_none() {
        assert_eq!(usage(r#"{"session_id":"abc-1"}"#), None);
        assert_eq!(usage(r#"{"rate_limits":null}"#), None);
        assert_eq!(usage(r#"{"rate_limits":{}}"#), None);
        assert_eq!(usage(r#"{"rate_limits":{"five_hour":{}}}"#), None);
        assert_eq!(
            usage(r#"{"rate_limits":{"five_hour":{"used_percentage":"9"}}}"#),
            None
        );
        assert_eq!(usage("not json"), None);
        assert_eq!(usage(""), None);
        let directory = scratch();
        assert_eq!(usage_in(&directory), None);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn usage_serializes_in_camel_case() {
        let result =
            usage(r#"{"rate_limits":{"five_hour":{"used_percentage":37,"resets_at":1700000000}}}"#)
                .unwrap();
        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["fiveHour"]["pct"], 37);
        assert_eq!(value["fiveHour"]["resetsAt"], 1_700_000_000u64);
        assert!(value["sevenDay"].is_null());
    }
}
