# Graph Report - crawde  (2026-11-03)

## Corpus Check
- 70 files · ~70,217 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 528 nodes · 964 edges · 43 communities (29 shown, 14 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS · INFERRED: 3 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `5e01d2fd`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- AppState
- .apply
- App.tsx
- updates.rs
- Changelog
- navigation.rs
- README.md
- types.ts
- tauri.conf.json
- lifecycle.rs
- compilerOptions
- bridge.rs
- devDependencies
- useBehavior.ts
- preferences.rs
- notifications.rs
- output
- dependencies
- default.json
- scripts
- check-packaged-assets.mjs
- platform.rs
- read_title
- Hooks
- Crawde v0.1.2
- Crawde v0.1.3
- PLAN.md
- v0.1.10.md
- v0.1.11.md
- v0.1.12.md
- v0.1.13.md
- v0.1.14.md
- v0.1.4.md
- v0.1.5.md
- v0.1.6.md
- v0.1.7.md
- v0.1.8.md
- v0.1.9.md
- make-branding.mjs
- crawde

## God Nodes (most connected - your core abstractions)
1. `Changelog` - 33 edges
2. `AppState` - 23 edges
3. `event()` - 16 edges
4. `Session` - 14 edges
5. `Activity` - 12 edges
6. `install()` - 12 edges
7. `compilerOptions` - 12 edges
8. `open_wsl()` - 11 edges
9. `open_chat()` - 11 edges
10. `new_chat()` - 11 edges

## Surprising Connections (you probably didn't know these)
- `AppState` --references--> `Activity`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/activity.rs
- `AppState` --references--> `Lifecycle`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/lifecycle.rs
- `AppState` --references--> `Navigation`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/navigation.rs
- `AppState` --references--> `Notifications`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/notifications.rs
- `AppState` --references--> `UpdateInfo`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/updates.rs

## Import Cycles
- None detected.

## Communities (43 total, 14 thin omitted)

### Community 0 - "AppState"
Cohesion: 0.11
Nodes (40): AtomicBool, Monitor, Mutex, PhysicalPosition, Session, Snapshot, AppState, dismiss_chat() (+32 more)

### Community 1 - ".apply"
Cohesion: 0.13
Nodes (28): BTreeSet, Activity, agents_are_deduplicated_and_approval_clears_on_tool_progress(), completion_expires_and_new_prompt_revives(), dismissed_chat_stays_hidden_through_tool_events_until_new_prompt(), event(), finished_time_is_not_counted_and_a_new_prompt_resumes_totals(), HookEvent (+20 more)

### Community 2 - "App.tsx"
Cohesion: 0.12
Nodes (22): license, name, private, type, version, App(), command(), native (+14 more)

### Community 3 - "updates.rs"
Cohesion: 0.17
Nodes (29): Path, activate(), Asset, asset_url(), check_on_launch(), check_updates(), client(), install() (+21 more)

### Community 4 - "Changelog"
Cohesion: 0.06
Nodes (33): 0.1.0 — 2026-10-26, 0.1.10 — 2026-10-27, 0.1.11 — 2026-10-27, 0.1.12 — 2026-10-27, 0.1.13 — 2026-10-27, 0.1.14 — 2026-10-27, 0.1.15 — 2026-10-27, 0.1.16 — 2026-10-27 (+25 more)

### Community 5 - "navigation.rs"
Cohesion: 0.21
Nodes (28): Duration, ExitStatus, claude_exe(), desktop_handoff_has_a_console_and_preserves_the_selected_chat(), desktop_handoff_has_a_terminal_and_preserves_the_selected_chat(), folder(), launch_desktop(), launch_terminal() (+20 more)

### Community 6 - "README.md"
Cohesion: 0.07
Nodes (24): Automated coverage, Sources, Validation, Windows release checklist, A crab on your desktop, Build, Checks, Claude integration (+16 more)

### Community 7 - "types.ts"
Cohesion: 0.18
Nodes (18): ChatBubbles(), Props, formatDuration(), sessionTiming(), session, shouldSleep(), SLEEP_AFTER_MS, session (+10 more)

### Community 8 - "tauri.conf.json"
Cohesion: 0.09
Nodes (21): icons/icon.ico, icons/icon.png, nsis, app, macOSPrivateApi, security, windows, build (+13 more)

### Community 9 - "lifecycle.rs"
Cohesion: 0.28
Nodes (19): Box, Error, Menu, close_settings(), context_menu(), handle_menu(), quit(), refresh_menu() (+11 more)

### Community 10 - "compilerOptions"
Cohesion: 0.10
Nodes (19): DOM, DOM.Iterable, ES2022, src, vite/client, vite.config.ts, compilerOptions, esModuleInterop (+11 more)

### Community 11 - "bridge.rs"
Cohesion: 0.23
Nodes (19): config_dir(), configure_hooks(), forward_hook(), install_is_idempotent_and_uninstall_preserves_other_hooks(), installed(), is_our_handler(), is_ours(), merge_hooks() (+11 more)

### Community 12 - "devDependencies"
Cohesion: 0.12
Nodes (17): devDependencies, @resvg/resvg-js, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite, @vitejs/plugin-react (+9 more)

### Community 13 - "useBehavior.ts"
Cohesion: 0.26
Nodes (11): Animation, animations, attackEffect(), attackEffects, clickAnimation(), framePosition(), randomAttack(), scheduleSleepExcursions() (+3 more)

### Community 14 - "preferences.rs"
Cohesion: 0.29
Nodes (13): Default, Self, field_updates_preserve_other_preferences(), load(), patched(), persist(), Preferences, AppHandle (+5 more)

### Community 15 - "notifications.rs"
Cohesion: 0.27
Nodes (11): Notifications, pcm_wave_has_valid_lengths_and_mutable_volume(), play(), AppHandle, BTreeMap, Preferences, String, Vec (+3 more)

### Community 16 - "output"
Cohesion: 0.27
Nodes (8): configure(), output(), quiet_command(), Command, Option, Result, String, Vec

### Community 17 - "dependencies"
Cohesion: 0.22
Nodes (9): dependencies, @phosphor-icons/react, react, react-dom, @tauri-apps/api, @phosphor-icons/react, react, react-dom (+1 more)

### Community 18 - "default.json"
Cohesion: 0.22
Nodes (8): core:default, pet, settings, description, identifier, permissions, $schema, windows

### Community 19 - "scripts"
Cohesion: 0.25
Nodes (8): scripts, branding, build, check, dev, preview, tauri, test

### Community 20 - "check-packaged-assets.mjs"
Cohesion: 0.25
Nodes (7): animationSource, bundle, config, directory, files, root, sprites

### Community 21 - "platform.rs"
Cohesion: 0.32
Nodes (6): app_data_dir(), home_dir(), Option, PathBuf, String, shell_quote()

### Community 22 - "read_title"
Cohesion: 0.57
Nodes (5): custom_title(), first_prompt(), read_title(), Option, String

### Community 23 - "Hooks"
Cohesion: 0.33
Nodes (3): Hooks, Run with Python 3 on Linux/WSL or Windows; never edits real Claude settings., skipUnless

### Community 24 - "Crawde v0.1.2"
Cohesion: 0.40
Nodes (4): Crawde v0.1.2, Fixed, Install, Updated

### Community 25 - "Crawde v0.1.3"
Cohesion: 0.40
Nodes (4): Crawde v0.1.3, Fixed, Install, Release checks

## Knowledge Gaps
- **145 isolated node(s):** `name`, `private`, `license`, `type`, `dev` (+140 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **14 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `AppState` connect `AppState` to `.apply`, `updates.rs`, `navigation.rs`, `notifications.rs`?**
  _High betweenness centrality (0.118) - this node is a cross-community bridge._
- **Why does `Activity` connect `.apply` to `AppState`?**
  _High betweenness centrality (0.061) - this node is a cross-community bridge._
- **Why does `UpdateInfo` connect `updates.rs` to `AppState`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **What connects `name`, `private`, `license` to the rest of the system?**
  _145 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `AppState` be split into smaller, more focused modules?**
  _Cohesion score 0.10782241014799154 - nodes in this community are weakly interconnected._
- **Should `.apply` be split into smaller, more focused modules?**
  _Cohesion score 0.13170731707317074 - nodes in this community are weakly interconnected._
- **Should `App.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.11948790896159317 - nodes in this community are weakly interconnected._