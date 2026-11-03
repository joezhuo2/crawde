use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize)]
pub struct HookEvent {
    #[serde(default)]
    pub wsl_distro: Option<String>,
    pub session_id: String,
    pub hook_event_name: String,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: serde_json::Value,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub notification_type: String,
    #[serde(default)]
    pub transcript_path: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub wsl_distro: Option<String>,
    pub id: String,
    pub cwd: String,
    pub project: String,
    pub title: String,
    pub status: String,
    pub activity: String,
    pub tool: String,
    pub agents: BTreeSet<String>,
    pub updated_at: u64,
    pub last_prompt_at: u64,
    pub finished_at: Option<u64>,
    pub working_ms: u64,
    pub waiting_ms: u64,
    pub status_since: u64,
}

impl Session {
    fn set_status(&mut self, status: &str, now: u64) {
        if self.status == status {
            return;
        }
        let elapsed = now.saturating_sub(self.status_since);
        match self.status.as_str() {
            "working" => self.working_ms += elapsed,
            "approval" => self.waiting_ms += elapsed,
            _ => {}
        }
        self.status = status.into();
        self.status_since = now;
    }
}

#[derive(Default)]
pub struct Activity {
    pub sessions: BTreeMap<String, Session>,
    pub last_prompt_at: u64,
    dismissed: BTreeSet<String>,
    pub last_activity_at: u64,
    pub last_event_at: u64,
}

pub fn short(text: &str, max: usize) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max)
        .collect()
}

impl Activity {
    pub fn apply(&mut self, event: HookEvent, now: u64) -> Result<(), String> {
        if event
            .wsl_distro
            .as_deref()
            .is_some_and(|distro| !crate::wsl::valid_distro(distro))
        {
            return Err("Invalid WSL distro".into());
        }
        if event.session_id.len() > 128
            || event.session_id.is_empty()
            || !event
                .session_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err("Invalid session id".into());
        }
        let id = event.session_id.clone();
        self.last_event_at = now;
        if matches!(
            event.hook_event_name.as_str(),
            "SessionStart"
                | "UserPromptSubmit"
                | "PreToolUse"
                | "PostToolUse"
                | "SubagentStart"
                | "SubagentStop"
                | "PermissionRequest"
                | "Elicitation"
        ) {
            self.last_activity_at = now;
        }
        if event.hook_event_name == "UserPromptSubmit" {
            self.last_prompt_at = now;
            self.dismissed.remove(&id);
        }
        if event.hook_event_name == "SessionEnd" {
            if let Some(session) = self.sessions.get_mut(&id) {
                session.set_status("done", now);
                session.activity = "✓ Session ended".into();
                session.finished_at = Some(now);
                session.agents.clear();
            }
            return Ok(());
        }
        // Bound memory even if a local producer sends many sessions.
        if !self.sessions.contains_key(&id) && self.sessions.len() >= 100 {
            if let Some(oldest) = self
                .sessions
                .values()
                .min_by_key(|s| s.updated_at)
                .map(|s| s.id.clone())
            {
                self.sessions.remove(&oldest);
            }
        }
        let session = self.sessions.entry(id.clone()).or_insert_with(|| Session {
            wsl_distro: event.wsl_distro.clone(),
            id,
            cwd: event.cwd.clone(),
            project: event
                .cwd
                .trim_end_matches(['/', '\\'])
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or("Claude")
                .into(),
            title: "New conversation".into(),
            status: "working".into(),
            activity: "💭 Getting ready".into(),
            tool: String::new(),
            agents: BTreeSet::new(),
            updated_at: now,
            last_prompt_at: now,
            finished_at: None,
            working_ms: 0,
            waiting_ms: 0,
            status_since: now,
        });
        session.updated_at = now;
        match event.hook_event_name.as_str() {
            "UserPromptSubmit" => {
                if session.title == "New conversation" {
                    let prompt_title = short(&event.prompt, 90);
                    if !prompt_title.is_empty() {
                        session.title = prompt_title;
                    }
                }
                session.last_prompt_at = now;
                session.set_status("working", now);
                session.finished_at = None;
                session.activity = "💭 Thinking".into();
                session.tool.clear();
                session.agents.clear();
            }
            "PreToolUse" | "PostToolUse" => {
                session.set_status("working", now);
                session.finished_at = None;
                session.tool = short(&event.tool_name, 80);
                let label = match event.tool_name.as_str() {
                    "Read" | "Glob" | "Grep" => "📚 Reading",
                    "Edit" | "Write" | "NotebookEdit" => "✏️ Editing",
                    "Bash" | "PowerShell" => "🔧 Running a command",
                    "WebSearch" | "WebFetch" => "🌐 Researching",
                    "Agent" | "Task" => "🤝 Delegating",
                    _ => "🛠️ Using a tool",
                };
                let detail = event
                    .tool_input
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                session.activity = if detail.is_empty() {
                    label.into()
                } else {
                    format!("{label} · {}", short(detail, 90))
                };
            }
            "PermissionRequest" | "Elicitation" => {
                session.set_status("approval", now);
                session.activity = "✋ Waiting for your approval".into();
                session.tool = short(&event.tool_name, 80);
            }
            "Notification" if event.notification_type == "permission_prompt" => {
                session.set_status("approval", now);
                session.activity = "✋ Waiting for your approval".into();
            }
            "SubagentStart" => {
                session.agents.insert(event.agent_id);
            }
            "SubagentStop" => {
                session.agents.remove(&event.agent_id);
            }
            "PostToolUseFailure" | "StopFailure" => {
                session.set_status("error", now);
                session.activity = "⚠️ Something needs attention".into();
            }
            "Stop" => {
                session.set_status("done", now);
                session.activity = "✓ Finished".into();
                session.finished_at = Some(now);
                session.agents.clear();
                session.tool.clear();
            }
            _ => {}
        }
        Ok(())
    }

    pub fn visible(&mut self, now: u64) -> Vec<Session> {
        self.sessions.retain(|_, s| {
            s.finished_at
                .is_none_or(|at| now.saturating_sub(at) < 30_000)
        });
        self.dismissed.retain(|id| self.sessions.contains_key(id));
        let mut sessions: Vec<_> = self
            .sessions
            .values()
            .filter(|s| !self.dismissed.contains(&s.id))
            .cloned()
            .collect();
        sessions.sort_by_key(|s| (s.status != "approval", std::cmp::Reverse(s.updated_at)));
        sessions
    }

    pub fn dismiss(&mut self, id: &str) {
        if self.sessions.contains_key(id) {
            self.dismissed.insert(id.to_string());
        }
    }

    pub fn is_dismissed(&self, id: &str) -> bool {
        self.dismissed.contains(id)
    }

    pub fn restore_dismissed(&mut self) {
        self.dismissed.clear();
    }

    pub fn has_recent_work(&self, now: u64) -> bool {
        self.sessions.values().any(|session| {
            session.status != "done" && now.saturating_sub(session.updated_at) < 600_000
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(name: &str) -> HookEvent {
        serde_json::from_value(
            serde_json::json!({"session_id":"test-1", "hook_event_name":name,"cwd":"C:\\project"}),
        )
        .unwrap()
    }
    #[test]
    fn wsl_events_preserve_linux_context_and_wake_the_pet() {
        let mut state = Activity::default();
        let mut input = event("UserPromptSubmit");
        input.wsl_distro = Some("Ubuntu-24.04".into());
        input.cwd = "/home/joe/My project".into();
        input.prompt = "Fix Linux integration".into();
        state.apply(input, 100).unwrap();
        let sessions = state.visible(101);
        assert_eq!(sessions[0].wsl_distro.as_deref(), Some("Ubuntu-24.04"));
        assert_eq!(sessions[0].cwd, "/home/joe/My project");
        assert_eq!(sessions[0].project, "My project");
        assert_eq!(sessions[0].title, "Fix Linux integration");
        assert!(state.has_recent_work(101));
    }
    #[test]
    fn notifications_restore_dismissed_chats_without_reviving_expired_sessions() {
        let mut state = Activity::default();
        state.apply(event("UserPromptSubmit"), 1).unwrap();
        state.dismiss("test-1");
        assert!(state.visible(2).is_empty());
        state.restore_dismissed();
        assert_eq!(state.visible(3).len(), 1);
        state.apply(event("Stop"), 4).unwrap();
        state.dismiss("test-1");
        state.restore_dismissed();
        assert!(state.visible(30_004).is_empty());
    }
    #[test]
    fn dismissed_chat_stays_hidden_through_tool_events_until_new_prompt() {
        let mut state = Activity::default();
        state.apply(event("UserPromptSubmit"), 1).unwrap();
        state.dismiss("test-1");
        state.apply(event("PermissionRequest"), 2).unwrap();
        assert!(state.visible(2).is_empty());
        assert!(state.is_dismissed("test-1"));
        state.apply(event("UserPromptSubmit"), 3).unwrap();
        assert_eq!(state.visible(3).len(), 1);
    }
    #[test]
    fn completion_expires_and_new_prompt_revives() {
        let mut state = Activity::default();
        state.apply(event("Stop"), 100).unwrap();
        assert_eq!(state.visible(30_099).len(), 1);
        state.apply(event("UserPromptSubmit"), 200).unwrap();
        assert_eq!(state.visible(40_000).len(), 1);
        state.apply(event("Stop"), 40_000).unwrap();
        assert!(state.visible(70_000).is_empty());
    }
    #[test]
    fn tool_events_wake_activity_without_a_prompt_and_session_end_clears_work() {
        let mut state = Activity::default();
        state.apply(event("PreToolUse"), 100).unwrap();
        assert_eq!(state.last_activity_at, 100);
        assert!(state.has_recent_work(100));
        state.apply(event("SessionEnd"), 200).unwrap();
        assert!(!state.has_recent_work(200));
        assert_eq!(state.last_event_at, 200);
    }
    #[test]
    fn agents_are_deduplicated_and_approval_clears_on_tool_progress() {
        let mut state = Activity::default();
        let mut start = event("SubagentStart");
        start.agent_id = "a".into();
        state.apply(start.clone(), 1).unwrap();
        state.apply(start, 2).unwrap();
        assert_eq!(state.visible(2)[0].agents.len(), 1);
        state.apply(event("PermissionRequest"), 3).unwrap();
        assert_eq!(state.visible(3)[0].status, "approval");
        state.apply(event("PostToolUse"), 4).unwrap();
        assert_eq!(state.visible(4)[0].status, "working");
    }
    #[test]
    fn working_and_waiting_time_accumulate_across_status_changes() {
        let mut state = Activity::default();
        state.apply(event("UserPromptSubmit"), 1_000).unwrap();
        state.apply(event("PermissionRequest"), 4_000).unwrap();
        state.apply(event("PostToolUse"), 9_000).unwrap();
        state.apply(event("Stop"), 12_000).unwrap();
        let session = &state.visible(12_000)[0];
        assert_eq!(session.working_ms, 6_000);
        assert_eq!(session.waiting_ms, 5_000);
        assert_eq!(session.status_since, 12_000);
        assert_eq!(session.updated_at, 12_000);
    }
    #[test]
    fn repeated_events_in_one_status_keep_the_running_segment() {
        let mut state = Activity::default();
        state.apply(event("UserPromptSubmit"), 100).unwrap();
        state.apply(event("PreToolUse"), 400).unwrap();
        state.apply(event("PostToolUse"), 700).unwrap();
        let session = &state.visible(700)[0];
        assert_eq!(session.working_ms, 0);
        assert_eq!(session.status_since, 100);
        assert_eq!(session.updated_at, 700);
    }
    #[test]
    fn finished_time_is_not_counted_and_a_new_prompt_resumes_totals() {
        let mut state = Activity::default();
        state.apply(event("UserPromptSubmit"), 0).unwrap();
        state.apply(event("Stop"), 2_000).unwrap();
        state.apply(event("UserPromptSubmit"), 10_000).unwrap();
        state.apply(event("SessionEnd"), 13_000).unwrap();
        let session = &state.visible(13_000)[0];
        assert_eq!(session.working_ms, 5_000);
        assert_eq!(session.waiting_ms, 0);
    }
    #[test]
    fn session_serializes_timing_fields_in_camel_case() {
        let mut state = Activity::default();
        state.apply(event("UserPromptSubmit"), 5).unwrap();
        let value = serde_json::to_value(&state.visible(5)[0]).unwrap();
        assert_eq!(value["workingMs"], 0);
        assert_eq!(value["waitingMs"], 0);
        assert_eq!(value["statusSince"], 5);
        assert_eq!(value["updatedAt"], 5);
    }
}
