<p align="center"><img src="assets/crawde-banner.svg" alt="Crawde — a little company while Claude works" width="100%"></p>

<p align="center">
  <a href="https://github.com/joezhuo2/crawde/releases/latest"><img src="https://img.shields.io/github/v/release/joezhuo2/crawde?logo=github&color=orange" alt="release"></a>
  <a href="https://github.com/joezhuo2/crawde/releases"><img src="https://img.shields.io/github/downloads/joezhuo2/crawde/total?logo=github&color=blue" alt="downloads"></a>
  <a href="https://github.com/joezhuo2/crawde/actions/workflows/windows.yml"><img src="https://img.shields.io/github/actions/workflow/status/joezhuo2/crawde/windows.yml?branch=main&label=CI&logo=githubactions&logoColor=white" alt="CI"></a>
  <img src="https://img.shields.io/badge/platform-Windows-lightgrey" alt="platform: Windows">
  <a href="https://v2.tauri.app"><img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2"></a>
  <a href="https://react.dev"><img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white" alt="React 19"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/joezhuo2/crawde?color=green" alt="license: MIT"></a>
</p>

<p align="center">
  <a href="#download">Download</a> •
  <a href="#features">Features</a> •
  <a href="#settings-and-keybinds">Keybinds</a> •
  <a href="#claude-integration">Claude integration</a> •
  <a href="#data">Data</a> •
  <a href="#files">Files</a> •
  <a href="#development">Development</a> •
  <a href="#contributing">Contributing</a> •
  <a href="#license">License</a>
</p>

> A small desktop companion built with Tauri 2 and React for Windows. A pixel crab
> sits above your other windows while Claude Code works. Hover over it to follow
> your chats, see the current tool or running subagents, and catch approvals and
> finished tasks. Click a chat to open it, or start one in the same project. The
> crab takes little walks, reacts to clicks, and drifts off when things go quiet.

## Download

Get the latest Windows x64 installer from [Releases](https://github.com/joezhuo2/crawde/releases/latest):

- **Windows:** `Crawde_x.y.z_x64-setup.exe`.
- A `.sha256` checksum is included with each release.
- Requires WebView2 and the [native Claude Code CLI](https://code.claude.com/docs/en/setup).

### First launch

1. Open Crawde, then right-click the crab or its tray icon and choose **Settings**.
2. In **Claude**, select **Connect activity hooks**.
3. Start a new Claude Code session, or restart an existing one so it loads the hooks.

Supports Claude Code in your terminal and the **Code tab** in Claude Desktop.
Regular Claude Chat, WSL/remote activity forwarding, and macOS are outside this version's scope.

### Updates

Crawde checks GitHub releases when it opens, unless you turn that off in settings.
**Settings → About → Check for updates** checks immediately. Downloads and installation
are manual; an update check does not replace or restart the app.

## Features

### A crab on your desktop

- **Always on top:** an animated pixel crab with a separate, readable tray icon.
- **Click and drag:** click for its ability animation; drag to move it. It snaps to
  all four screen edges and remembers positions for each monitor arrangement.
- **Little walks:** occasional short runs within roughly 100 pixels of its saved
  spot. Hovering or keeping chats expanded pauses roaming.
- **Sleep:** after ten minutes without activity, the crab slows down and little
  “zzz” letters drift away. Tools and subagents keep it awake while Claude works.
- **Reduced motion:** respects Windows preferences and Crawde's own setting.

### Follow your chats

- **Chat bubbles:** overlapping titles when collapsed; hover to expand the list.
  A pin keeps it open, and three chats fit per page with soft page fades.
- **Current activity:** project and conversation names, Claude's current step,
  and either the running subagent count or current tool.
- **Approvals:** a gentle orange tint, occasional claw strikes, and an optional sound.
- **Completion:** a checkmark and sound; finished cards leave after thirty seconds.
- **Dismiss:** each card has an ×. It stays hidden until that chat receives a new prompt.
- **Open or create:** click a card to resume its conversation. The plus button starts
  a chat in the selected project, falling back to the most recently active project.

## Settings and keybinds

Right-click the crab or its tray icon and choose **Settings**. The menu stays small;
preferences live in the settings window.

| Action | Control |
| --- | --- |
| Show or hide Crawde | Tray click or **Ctrl+Alt+P** by default; configurable |
| Expand chats | Hover over the crab or a card |
| Keep the list open | Pin button or the bell below the crab |
| Open a conversation | Click its card |
| Start a chat in the current project | Plus button below the crab |
| Dismiss a card | Its × button |
| Play the crab ability | Click the crab |
| Move the crab | Drag it |
| Quit | **Settings → About → Quit Crawde** |

Settings include system/light/dark appearance, integer pet sizes, roaming, reduced
motion, the global hotkey, and launch with Windows. Sounds have volume, soft/retro
styles, separate event toggles, and approval reminders. Chat navigation can use
Claude Desktop or Windows Terminal.

Interactive launches show the pet even if Claude is closed. Startup launches wait
in the tray until Claude opens. Windows may put the tray icon in its overflow menu.

## Claude integration

**Settings → Claude** shows whether hooks are installed and when activity was last
received. If it says no activity has arrived, restart the Claude session after
connecting hooks and check that it is running locally on Windows.

Crawde preserves your existing Claude hooks, backs up settings before changes,
and removes only its own handlers when disconnected. Older startup-hook
configurations are repaired automatically when Crawde launches. It observes work;
it never approves or denies Claude's actions.

<details>
<summary>How activity and chat navigation work</summary>

`SessionStart` uses a quiet `crawde.exe --crawde-hook` command that forwards stdin
to the local receiver. Claude does not support HTTP handlers for this event. The
other tracked events use HTTP hooks. The receiver binds `127.0.0.1:47631`, requires
a local bearer token, rejects browser-origin requests, and limits payloads to 64 KiB.

Desktop navigation uses `claude --desktop --resume <id>`. Terminal navigation
resumes in a new tab rather than focusing an already running tab. New Desktop chats
use a folder-aware deep link, so Claude may ask you to confirm the project folder.

</details>

## Data

Live session metadata stays in memory. Crawde reads bounded sections of matching
local Claude transcripts to recover conversation names. It does not send prompts,
tool arguments, or transcripts to an external service. Update checks contact the
public GitHub releases endpoint.

## Files

| What | Windows |
| --- | --- |
| Preferences | `%APPDATA%\com.joezhuo.crawde\preferences.json` |
| Monitor positions | `%APPDATA%\com.joezhuo.crawde\positions.json` |
| Local bridge token | `%APPDATA%\com.joezhuo.crawde\bridge-token` |
| Claude hooks | `%USERPROFILE%\.claude\settings.json`, or `CLAUDE_CONFIG_DIR` |
| Hook backups | `settings.crawde-<timestamp>.bak` beside Claude's settings |

## Development

### Build

Requirements: Node.js 24+, Rust 1.90+, Visual Studio C++ Build Tools, WebView2,
and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

```powershell
npm ci
npm run tauri dev       # desktop app with hot reload
npm run tauri build     # Windows NSIS installer
npm run branding        # regenerate the banner and dedicated tray artwork
```

The installer is written to `src-tauri/target/release/bundle/nsis/`.
Development launches do not automatically register a Windows startup entry.

For an explicit UI preview, run `npm run dev` and open
`http://127.0.0.1:1420/?demo`, `?settings&demo`, or `?settings&loading`.
Demo data is never used as a native connection fallback.

### Checks

```powershell
npm run check
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
./scripts/bridge-smoke.ps1  # with Crawde running; uses temporary test cards
```

See [validation notes](docs/VALIDATION.md) for coverage and hardware checks.
The [Windows workflow](.github/workflows/windows.yml) validates pushes and pull
requests and uploads an installer artifact.

### Releases

Update the version in `package.json`, `src-tauri/Cargo.toml`, and
`src-tauri/tauri.conf.json`, refresh both lockfiles, and add a
[changelog](CHANGELOG.md) entry. Push a version tag to run the
[release workflow](.github/workflows/release.yml), which validates the tagged
source, builds the installer, generates release notes, and attaches the installer
and checksum to its GitHub release.

## Contributing

Bug reports should include your Windows version, Crawde version, whether you use
Claude Desktop or the terminal, and the connection status shown in **Settings → Claude**.
Keep tokens and private conversation contents out of reports.

## License

Application code and the Crawde branding assets are [MIT licensed](LICENSE).
The supplied crab sprite sheets retain their original terms; see the
[sprite notice](sprites/NOTICE.md).
