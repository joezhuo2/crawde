# Validation

## Automated coverage

- Sleep outing timing, attack counts, return to sleep, and timer cancellation.
- Equal-probability selection of all three click animations.
- Attack FX use the reference strike frames (attack 1: 7–9, attack 2: 3–5, attack 3: 5–7 and 8–10), with no effects outside those frames.
- Sprite row/column addressing and nonempty frame counts.
- Session completion expiry at 30 seconds and revival on a new prompt.
- Subagent deduplication and approval-to-tool transitions.
- Idempotent hook installation and preservation of unrelated hooks on disconnect.
- Middle-screen drop coordinates survive transparent-window repositioning; negative monitor coordinates, DPI scaling, off-screen bounds, and saved-anchor migration.
- Separate walk cadences for active work, idle, and temporary wake-ups (3–7 seconds), bounded offsets around the placed position, and turning back at window edges.
- WAV structure and volume scaling.
- Folder URL encoding and transcript title parsing.
- Wake-up from tool activity without process detection, continued activity after an old prompt, and sleep after inactivity.
- Migration from unsupported HTTP startup hooks to command forwarding without replacing unrelated handlers.
- Production bundles contain all seven sprite sheets as local assets and no inline images blocked by the native Content Security Policy. This check runs during every build, including release packaging.

## Windows release checklist

- [ ] Installer builds and launches on Windows.
- [ ] First-launch settings render; install/reinstall hooks from the right-click menu without replacing existing settings.
- [ ] CLI and Desktop Code events appear with correct titles and current tool/subagent details.
- [ ] Hover expands, pin persists while leaving, three-chat pagination works.
- [ ] Completion checkmark expires after 30 seconds; new prompts restore the chat.
- [ ] Attack 1/2/3 sparks match CrabComplete.gif at both facing directions and all scales, without clipping; reduced motion hides FX.
- [ ] Ability, run, idle, attacks, sleep, and reduced motion render correctly.
- [ ] Walking moves only the crab; pinned session cards stay anchored and the moving crab remains clickable.
- [ ] Sleeping crab wakes after 1–4 minutes, walks and attacks 1–3 times over 15–45 seconds, then sleeps; wake-up walks pause 3–7 seconds and never overlap attacks; new activity and motion preferences cancel outings.
- [ ] Walks pause during hover, dragging, keyboard focus, sleep, and reduced motion.
- [ ] While Claude works, the crab attacks (A, B or C) every 8–14 seconds only when idle and not walking; walking never starts mid-attack.
- [ ] Crab has no rectangular outline on click, focus, or drag; keyboard activation still works and other controls retain focus indicators.
- [ ] Drop at 20%, 35%, 50%, 65%, and 80% screen height in both directions: crab stays at the drop point; expanding cards or changing direction does not move it. Restart restores that position.
- [ ] Overflowing cards scroll within the available space and clipped controls do not intercept clicks.
- [ ] Drag versus click is distinct; near-edge placement stays put, off-screen positions are corrected, chats expand down/up in the top/bottom half; display positions survive restart.
- [ ] Transparent areas pass clicks through; visible controls remain usable.
- [ ] Tray and hotkey toggle visibility; both right-click menus offer Settings, preference toggles, hook actions, update actions, and Quit.
- [ ] Moved controls appear only in menus; tray checkmarks and the pet reflect saved state after restart.
- [ ] Settings save automatically, including rapid changes; startup registration follows the startup preference.
- [ ] Native sound playback works before the first pet click; mute and volume work.
- [ ] Clicking a card resumes the correct conversation; no new-chat or notification buttons remain beneath the crab.
- [ ] Install update verifies the checksum, launches the installer before exiting, and stays open on failure.
- [ ] Both menus show one versioned update action; available updates install on click and checks run on launch even with an old disabled preference.
- [ ] Update check reports latest version, network errors, and no-release state honestly.

Multi-monitor hotplug/DPI changes and the installed startup flow need testing on representative Windows hardware. Public release download detection requires a published GitHub release. Terminal chat navigation resumes in a new tab instead of focusing an existing one.

## Sources

- [Tauri 2 tray](https://v2.tauri.app/learn/system-tray/), [shortcuts](https://v2.tauri.app/plugin/global-shortcut/), [autostart](https://v2.tauri.app/plugin/autostart/).
- [Claude hook reference](https://code.claude.com/docs/en/hooks).
- [Claude Desktop deep links](https://support.claude.com/en/articles/14729294-open-claude-desktop-with-a-link).
- Local installed `claude --help` confirms `--desktop --resume` support.
