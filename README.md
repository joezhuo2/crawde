# Crawde

A tiny animated crab that keeps you company while Claude Code works. Windows · Tauri 2 · React.

## Run

Install Node.js 24+, Rust 1.90+, Visual Studio C++ Build Tools, WebView2, and the **native Claude Code CLI**.

```powershell
npm ci
npm run tauri dev
```

On first launch, open **Claude → Connect activity hooks** in Crawde's settings, then restart existing Claude sessions. Supports Claude Code in the terminal and the **Code tab** in Claude Desktop. Regular Claude Chat and WSL/remote sessions are outside this version's scope.

Build a Windows installer:

```powershell
npm run tauri build
```

Installer: `src-tauri/target/release/bundle/nsis/Crawde_0.1.0_x64-setup.exe`.
The Windows GitHub Actions workflow also uploads the installer as a build artifact.

## Use

- **Hover:** expand the overlapping chat bubbles. Three chats fit on a page; arrows reveal the rest.
- **Pin:** keep the conversation list expanded.
- **Click a bubble:** open its conversation. Select Claude Desktop or Windows Terminal in settings.
- **Click the crab:** play its ability. **Drag:** move it; it snaps to screen edges and remembers positions per monitor.
- **Plus:** create a chat in the last selected project, falling back to the most recently active project.
- **Bell:** keep the chat list open or tuck it away when the pointer leaves.
- **Ctrl+Alt+P** or **tray click:** show/hide Crawde. The hotkey is configurable.
- **Right-click pet or tray → Settings:** appearance, startup, movement, sounds, integration, updates, and Quit.

The installed app waits in the tray on Windows startup and appears when Claude opens. If Claude closes, Crawde sleeps. It also sleeps after ten minutes without a prompt. Finished chat bubbles disappear after thirty seconds. Approvals tint their card orange and occasionally trigger an attack until Claude reports progress. Sounds distinguish approval, completion, and error; optional approval reminders repeat every forty-five seconds.

Sprites are rendered with integer scaling and nearest-neighbor pixels. Reduced motion respects both Windows settings and Crawde's own preference. Development launches do not automatically register a startup entry.

## Integration and data

Crawde adds observational HTTP hooks to `%USERPROFILE%/.claude/settings.json` (or `CLAUDE_CONFIG_DIR`). It preserves existing handlers, creates a timestamped backup on each change, and removes only its own handlers when disconnected. Existing sessions may need restarting after hook changes. Hooks never approve or deny Claude actions.

The receiver binds **127.0.0.1:47631**, requires a locally generated bearer token, rejects browser-origin requests, and limits payloads to 64 KiB. It keeps live session metadata in memory. It reads bounded sections of matching local Claude transcripts to recover conversation names. Prompts, tool arguments, and transcripts are not sent to any external service.

Preferences, monitor positions, and the bridge token live in `%APPDATA%/com.joezhuo.crawde`. Update checks contact the public GitHub releases endpoint; downloads and installation are manual. No release published yet is reported explicitly.

Chat navigation requires the native `claude.exe`. Desktop navigation uses `claude --desktop --resume <id>`; terminal navigation opens a resume tab. This does not focus an already running terminal tab. New Desktop chats use Claude's folder-aware deep link; Claude may ask you to confirm that folder.

## Validate

```powershell
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

With Crawde running, `./scripts/bridge-smoke.ps1` exercises authentication and representative lifecycle events using temporary test sessions. Those cards expire after the script ends. To preview the UI in a browser, run `npm run dev` and visit `http://127.0.0.1:1420/?demo` or `?settings&demo`. Demo data is opt-in and never used as native connection fallback.

Implementation notes and release checks: [docs/VALIDATION.md](docs/VALIDATION.md). Feature branches and their small commits are retained in GitHub; `main` integrates the complete app.
