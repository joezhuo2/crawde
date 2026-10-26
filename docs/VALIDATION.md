# Validation

## Automated coverage

- Sprite row/column addressing and nonempty frame counts.
- Session completion expiry at 30 seconds and revival on a new prompt.
- Subagent deduplication and approval-to-tool transitions.
- Idempotent hook installation and preservation of unrelated hooks on disconnect.
- Negative monitor coordinates, edge snapping, and smaller display bounds.
- WAV structure and volume scaling.
- Folder URL encoding and transcript title parsing.
- Wake-up from tool activity without process detection, continued activity after an old prompt, and sleep after inactivity.
- Migration from unsupported HTTP startup hooks to command forwarding without replacing unrelated handlers.

## Windows release checklist

- [ ] Installer builds and launches on Windows.
- [ ] First-launch settings render; connect hooks without replacing existing settings.
- [ ] CLI and Desktop Code events appear with correct titles and current tool/subagent details.
- [ ] Hover expands, pin persists while leaving, three-chat pagination works.
- [ ] Completion checkmark expires after 30 seconds; new prompts restore the chat.
- [ ] Ability, run, idle, attacks, sleep, and reduced motion render correctly.
- [ ] Drag versus click is distinct; all four edges snap; display positions survive restart.
- [ ] Transparent areas pass clicks through; visible controls remain usable.
- [ ] Tray and hotkey toggle visibility; only Settings appears in right-click menus.
- [ ] Settings persist; startup registration follows the startup preference.
- [ ] Native sound playback works before the first pet click; mute and volume work.
- [ ] New chat uses the selected project and clicking a card resumes the correct conversation.
- [ ] Update check reports latest version, network errors, and no-release state honestly.

Multi-monitor hotplug/DPI changes and the installed startup flow need testing on representative Windows hardware. Public release download detection requires a published GitHub release. Terminal chat navigation resumes in a new tab instead of focusing an existing one.

## Sources

- [Tauri 2 tray](https://v2.tauri.app/learn/system-tray/), [shortcuts](https://v2.tauri.app/plugin/global-shortcut/), [autostart](https://v2.tauri.app/plugin/autostart/).
- [Claude hook reference](https://code.claude.com/docs/en/hooks).
- [Claude Desktop deep links](https://support.claude.com/en/articles/14729294-open-claude-desktop-with-a-link).
- Local installed `claude --help` confirms `--desktop --resume` support.
