<p align="center"><img src="assets/crawde-banner.png" alt="Crawde — a little company while Claude works" width="100%"></p>

<p align="center">
  <a href="https://github.com/joezhuo2/crawde/releases/latest"><img src="https://img.shields.io/github/v/release/joezhuo2/crawde?logo=github&color=orange" alt="release"></a>
  <a href="https://github.com/joezhuo2/crawde/releases"><img src="https://img.shields.io/github/downloads/joezhuo2/crawde/total?logo=github&color=blue" alt="downloads"></a>
  <a href="https://github.com/joezhuo2/crawde/actions/workflows/windows.yml"><img src="https://img.shields.io/github/actions/workflow/status/joezhuo2/crawde/windows.yml?branch=main&label=CI&logo=githubactions&logoColor=white" alt="CI"></a>
  <img src="https://img.shields.io/badge/platform-Windows-lightgrey" alt="platform: Windows">
  <a href="https://v2.tauri.app"><img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2"></a>
  <a href="https://react.dev"><img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white" alt="React 19"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="license: MIT"></a>
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
> finished tasks. Click a chat to open it. The
> crab takes little walks, reacts to clicks, and drifts off when things go quiet.

## Download

Get the latest Windows x64 installer from [Releases](https://github.com/joezhuo2/crawde/releases/latest):

- **Windows:** `Crawde_x.y.z_x64-setup.exe`.
- A `.sha256` checksum is included with each release.
- Requires WebView2 and the [native Claude Code CLI](https://code.claude.com/docs/en/setup).

### First launch

1. Open Crawde, then right-click the crab or its tray icon.
2. Select **Install activity hooks**.
3. Start a new Claude Code session, or restart an existing one so it loads the hooks.

Supports Claude Code in your terminal and the **Code tab** in Claude Desktop.
Ubuntu/Linux Claude Code sessions in WSL on the same Windows PC are supported.
Separate Linux machines, regular Claude Chat, and macOS are outside this version's scope.

### Updates

Crawde always checks GitHub releases on launch. The crab and tray right-click menus
show one update action: **Up to date (vx.y.z)** with your installed version, or
**Install latest update (vx.y.z)** with the available version. Clicking the latter
downloads and verifies the installer, launches it, and closes Crawde. Clicking
**Up to date** checks again. Checks never install an update automatically.

While checking or downloading, the action shows progress and is disabled. Failures
leave Crawde running and offer a retry; the About tab also shows update status.

## Features

### A crab on your desktop

- **Always on top:** an animated pixel crab with a separate, high-resolution claw tray icon.
- **Click and drag:** click for its ability animation; drag to move it. It snaps to
  all four screen edges and remembers positions for each monitor arrangement.
- **Little walks:** only the crab wanders left and right within 100 pixels of its
  placed spot; session cards stay anchored. While Claude works, it pauses 4–8 seconds
  between walks (35–60 seconds when idle). Hovering, dragging, keyboard focus,
  sleep, and reduced motion pause roaming; pinned cards can stay open during walks.
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
- **Open:** click a card to resume its conversation.

## Settings and keybinds

Right-click the crab or its tray icon and choose **Settings**. The same menu provides
**Reduce motion**, **Launch on startup**, **Little walks**, **Sound effects**, activity
hook installation/reinstallation and disconnection, updates, and **Quit Crawde**.
These controls live only in the right-click menus and save immediately.

| Action | Control |
| --- | --- |
| Show or hide Crawde | Tray click or **Ctrl+Alt+P** by default; configurable |
| Expand chats | Hover over the crab or a card |
| Keep the list open | Pin button in the chat heading |
| Open a conversation | Click its card |
| Dismiss a card | Its × button |
| Open Claude and play the crab ability | Click the crab, or focus it and press Enter/Space |
| Move the crab | Drag it |
| Quit | **Right-click → Quit Crawde** |

Changes save automatically. The shortcut saves when you leave its field or press Enter.

Settings include system/light/dark appearance, integer pet sizes, and the global
hotkey. Sounds have volume, soft/retro
styles, separate event toggles, and approval reminders. Chat navigation can use
Claude Desktop or Windows Terminal.

Interactive launches show the pet even if Claude is closed. Startup launches wait
in the tray until Claude opens. Windows may put the tray icon in its overflow menu.

## Claude integration

**Settings → Claude** shows whether hooks are installed and when activity was last
received. Install or reinstall hooks from the right-click menu. If it says no activity has arrived, restart the Claude session after
connecting hooks. For WSL sessions, follow the setup below.

Crawde preserves your existing Claude hooks, backs up settings before changes,
and removes only its own handlers when disconnected. Older startup-hook
configurations are repaired automatically when Crawde launches. It observes work;
it never approves or denies Claude's actions.

### Ubuntu/Linux sessions in WSL

After upgrading to 0.1.8, choose **Reinstall activity hooks**. Crawde updates the
Windows configuration and existing `~/.claude` configurations in registered WSL
distros (excluding Docker's internal distros). Start Claude once in a new distro
before reinstalling hooks, then restart existing Claude sessions.

WSL setup requires Python 3, `wslpath`, and enabled Windows executable interop.
Every WSL event invokes the Windows Crawde executable, which forwards it to the
Windows loopback receiver. This works with WSL NAT and mirrored networking;
no firewall rule or network listener change is required. Hooks quietly return
when Crawde is closed. Chat cards launch Claude in the
original distro and project, using its interactive Bash environment. WSL chats
always open in a terminal, including when the Desktop preference is selected.

The distro's `CLAUDE_CONFIG_DIR` is honored when exported to the setup process;
otherwise setup uses the default user's `~/.claude`. Configurations belonging to
other Linux users require separate setup. Disconnect removes Crawde's hooks in
both Windows and WSL and preserves other handlers and timestamped backups.

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
