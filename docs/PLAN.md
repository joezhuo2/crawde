# Crawde implementation

Windows Tauri 2 companion for Claude Code CLI and the Claude Desktop Code tab.
Native Rust owns session events, lifecycle, settings, tray, shortcuts, and positioning.
React renders a transparent overlay with crisp sprites, stacked chat bubbles, and a separate settings window.

Feature branches are integrated with fast-forward merges. Each branch is pushed and retained.

1. `feat/tauri-shell`: application foundation.
2. `feat/sprite-animation`: frame metadata, idle, run, attacks, ability, sleep.
3. `feat/claude-activity`: authenticated local hook bridge and session state.
4. `feat/chat-bubbles`: stacked activity cards, expansion, pinning, pagination.
5. `feat/window-positioning`: dragging, edge snapping, per-monitor persistence, roaming.
6. `feat/tray-lifecycle`: tray, global shortcut, Claude presence, startup.
7. `feat/settings`: persisted preferences and compact settings UI.
8. `feat/sound-notifications`: event sounds, approval reminders, volume.
9. `feat/chat-navigation`: chat resume and project-aware new chats.
10. `feat/update-checks`: GitHub release checking and Windows CI.

No `/pet`, voice, or regular Claude Chat-tab monitoring. Hooks only observe;
they never grant or reject approvals. Finished cards expire after 30 seconds.
Sleep begins after 10 minutes without a prompt or when Claude is closed.
